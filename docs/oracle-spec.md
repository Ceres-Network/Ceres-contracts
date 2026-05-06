# Oracle Specification

This document specifies the oracle system for Ceres Network, including data sources, submission protocols, and aggregation methods.

## Overview

The Ceres Network oracle system provides reliable weather and crop health data for parametric insurance triggers. The system uses multiple oracle nodes submitting data from various sources, with median aggregation to ensure resilience against manipulation.

## Data Types

### 1. Rainfall

**Description**: Cumulative rainfall over a time period

**Unit**: Millimeters (mm)

**Sources**:
- Weather station networks (NOAA, national meteorological services)
- Satellite data (CHIRPS, GPM IMERG)
- Ground sensors (IoT weather stations)

**Typical Values**:
- Drought threshold: 150-250mm per season
- Normal season: 300-600mm
- Flood threshold: >800mm per season

**Update Frequency**: Daily aggregation

### 2. NDVI (Normalized Difference Vegetation Index)

**Description**: Measure of vegetation health and density

**Unit**: Dimensionless, scaled by 10000 (0.7 = 7000)

**Range**: 0-10000 (0.0-1.0)

**Sources**:
- Sentinel-2 satellite (10m resolution, 5-day revisit)
- Landsat 8/9 (30m resolution, 16-day revisit)
- MODIS (250m resolution, daily)

**Typical Values**:
- Bare soil: 1000-2000 (0.1-0.2)
- Sparse vegetation: 2000-4000 (0.2-0.4)
- Healthy crops: 6000-8000 (0.6-0.8)
- Dense vegetation: 8000-9000 (0.8-0.9)

**Update Frequency**: Weekly (cloud-free images)

**Calculation**:
```
NDVI = (NIR - Red) / (NIR + Red)
```

### 3. Soil Moisture

**Description**: Volumetric water content in soil

**Unit**: Percentage (%)

**Range**: 0-100%

**Sources**:
- SMAP satellite (36km resolution, 2-3 day revisit)
- SMOS satellite (40km resolution, 3-day revisit)
- Ground sensors (IoT soil moisture probes)

**Typical Values**:
- Wilting point: <15%
- Field capacity: 25-35%
- Saturation: >50%

**Update Frequency**: Every 2-3 days

## Geospatial Indexing

### Geohash System

Ceres uses [geohash](https://en.wikipedia.org/wiki/Geohash) for spatial indexing:

**Precision Levels**:
| Precision | Cell Size | Use Case |
|-----------|-----------|----------|
| 3 | ~156km × 156km | Regional aggregation |
| 4 | ~39km × 19.5km | District level |
| 5 | ~4.9km × 4.9km | **Farm level (default)** |
| 6 | ~1.2km × 0.6km | Field level |
| 7 | ~153m × 153m | Sub-field level |

**Default**: Precision 5 (~5km cell) balances:
- Spatial resolution (covers typical smallholder farm)
- Data availability (satellite pixels)
- Privacy (doesn't reveal exact farm location)

**Example Geohashes**:
- `9q5ct` - San Francisco, CA
- `s00twy` - Nairobi, Kenya
- `w21z7` - Mumbai, India

**Conversion**:
```python
import geohash2

# Encode lat/lon to geohash
gh = geohash2.encode(-1.2921, 36.8219, precision=5)  # "kzgvh"

# Decode geohash to lat/lon
lat, lon = geohash2.decode("kzgvh")  # (-1.2921, 36.8219)
```

## Oracle Node Architecture

### Node Requirements

**Hardware**:
- 2+ CPU cores
- 4GB+ RAM
- 100GB+ storage
- Stable internet connection

**Software**:
- Stellar SDK
- Data source API clients
- Signing key management
- Monitoring and alerting

### Node Responsibilities

1. **Data Collection**
   - Fetch data from multiple sources
   - Validate data quality
   - Handle API rate limits

2. **Data Processing**
   - Aggregate to geohash cells
   - Calculate derived metrics (NDVI)
   - Temporal aggregation (daily, weekly)

3. **Submission**
   - Sign readings with node private key
   - Submit to oracle contract
   - Retry on failure

4. **Monitoring**
   - Track submission success rate
   - Monitor data source availability
   - Alert on anomalies

### Data Source Integration

#### Rainfall: CHIRPS

**API**: Climate Hazards Group InfraRed Precipitation with Station data

**Endpoint**: `https://data.chc.ucsb.edu/api/`

**Resolution**: 0.05° (~5km)

**Latency**: 2-3 days

**Example Request**:
```bash
curl "https://data.chc.ucsb.edu/api/chirps/v2.0/daily/\
?lat=-1.2921&lon=36.8219&start=2024-01-01&end=2024-03-31"
```

**Processing**:
```python
def get_rainfall(lat, lon, start_date, end_date):
    data = fetch_chirps(lat, lon, start_date, end_date)
    total_rainfall = sum(day['precip'] for day in data)
    return int(total_rainfall)  # mm
```

#### NDVI: Sentinel-2

**API**: Copernicus Open Access Hub

**Endpoint**: `https://scihub.copernicus.eu/dhus/`

**Resolution**: 10m (resampled to geohash cell)

**Latency**: 1-2 days

**Example Request**:
```python
from sentinelsat import SentinelAPI

api = SentinelAPI(user, password)
products = api.query(
    area=geohash_to_bbox('9q5ct'),
    date=('20240101', '20240331'),
    platformname='Sentinel-2',
    cloudcoverpercentage=(0, 20)
)
```

**Processing**:
```python
def calculate_ndvi(red_band, nir_band):
    ndvi = (nir_band - red_band) / (nir_band + red_band)
    ndvi_scaled = int(ndvi * 10000)
    return ndvi_scaled
```

#### Soil Moisture: SMAP

**API**: NASA Earthdata

**Endpoint**: `https://n5eil01u.ecs.nsidc.org/`

**Resolution**: 36km

**Latency**: 1-2 days

**Example Request**:
```python
from smap_io import SMAP_L3

smap = SMAP_L3()
data = smap.read(
    lat=-1.2921,
    lon=36.8219,
    date='2024-01-15'
)
soil_moisture = int(data['soil_moisture'] * 100)  # percentage
```

## Submission Protocol

### Reading Structure

```rust
struct Reading {
    oracle_node: Address,      // Oracle's Stellar address
    geo_cell: String,          // Geohash (precision 5)
    reading_type: ReadingType, // Rainfall | NDVI | SoilMoisture
    value: u32,                // Scaled value
    timestamp: u64,            // Unix timestamp
}
```

### Signature Scheme

**Algorithm**: Ed25519 (Stellar native)

**Message Format**:
```
message = concat(
    oracle_node,
    geo_cell,
    reading_type,
    value,
    timestamp
)
signature = sign_ed25519(message, oracle_private_key)
```

**Verification**:
```rust
fn verify_signature(
    message: &[u8],
    signature: &[u8; 64],
    public_key: &Address
) -> bool {
    env.crypto().ed25519_verify(public_key, message, signature)
}
```

### Submission Flow

```
1. Oracle fetches data from sources
   ↓
2. Oracle aggregates to geohash cells
   ↓
3. Oracle signs reading
   ↓
4. Oracle calls oracle_contract.submit_reading()
   ↓
5. Contract validates:
   - Oracle is whitelisted
   - Reading age < 48 hours
   - Signature is valid
   ↓
6. Reading stored on-chain
```

### Error Handling

**Retry Logic**:
```python
def submit_with_retry(reading, max_retries=3):
    for attempt in range(max_retries):
        try:
            result = oracle_contract.submit_reading(reading)
            return result
        except NetworkError:
            if attempt < max_retries - 1:
                time.sleep(2 ** attempt)  # Exponential backoff
            else:
                raise
```

**Error Types**:
- `OracleNotWhitelisted`: Node not authorized
- `ReadingTooOld`: Timestamp > 48 hours old
- `InvalidSignature`: Signature verification failed
- `NetworkError`: RPC connection failed

## Aggregation Methods

### Median Calculation

**Why Median?**
- Resistant to outliers
- Requires majority collusion to manipulate
- More robust than mean for adversarial environments

**Algorithm**:
```rust
fn calculate_median(values: Vec<u32>) -> u32 {
    if values.is_empty() {
        return 0;
    }
    
    // Sort values
    let mut sorted = values.clone();
    sorted.sort();
    
    let len = sorted.len();
    if len % 2 == 0 {
        // Even: average of middle two
        (sorted[len/2 - 1] + sorted[len/2]) / 2
    } else {
        // Odd: middle value
        sorted[len/2]
    }
}
```

**Example**:
```
Readings: [150, 180, 200, 220, 500]
Sorted:   [150, 180, 200, 220, 500]
Median:   200 (middle value)

Without median (mean): 250 (skewed by outlier 500)
```

### Temporal Aggregation

**Season Window**:
```rust
fn aggregate_readings(
    geo_cell: String,
    reading_type: ReadingType,
    season_window: u64  // seconds
) -> AggregatedReading {
    let current_time = env.ledger().timestamp();
    let cutoff_time = current_time - season_window;
    
    // Get all readings within window
    let readings = get_readings_since(geo_cell, reading_type, cutoff_time);
    
    // Extract values
    let values: Vec<u32> = readings.iter().map(|r| r.value).collect();
    
    // Calculate median
    let median = calculate_median(values);
    
    AggregatedReading {
        geo_cell,
        reading_type,
        value: median,
        last_updated: current_time,
        sample_count: values.len(),
    }
}
```

**Window Sizes**:
- Rainfall: Full season (60-120 days)
- NDVI: Recent 30 days (crop health snapshot)
- Soil Moisture: Recent 14 days (current conditions)

## Data Quality Assurance

### Validation Rules

**Rainfall**:
- Range: 0-2000mm per season
- Reject if: value > 2000 (likely error)
- Reject if: sudden spike >500mm in 1 day

**NDVI**:
- Range: 0-10000 (0.0-1.0)
- Reject if: value > 10000
- Reject if: negative value

**Soil Moisture**:
- Range: 0-100%
- Reject if: value > 100
- Reject if: value changes >50% in 1 day

### Anomaly Detection

**Statistical Outliers**:
```python
def is_outlier(value, historical_values):
    mean = np.mean(historical_values)
    std = np.std(historical_values)
    z_score = abs((value - mean) / std)
    return z_score > 3  # 3 standard deviations
```

**Temporal Consistency**:
```python
def check_temporal_consistency(current, previous):
    max_change = {
        'Rainfall': 100,  # mm per day
        'NDVI': 1000,     # 0.1 change
        'SoilMoisture': 20  # 20% per day
    }
    change = abs(current - previous)
    return change <= max_change[reading_type]
```

### Oracle Reputation

**Metrics**:
- Submission frequency
- Data quality score
- Uptime percentage
- Deviation from consensus

**Reputation Score**:
```python
reputation = (
    0.3 * submission_frequency +
    0.3 * data_quality +
    0.2 * uptime +
    0.2 * consensus_alignment
)
```

**Actions**:
- Score < 0.5: Warning
- Score < 0.3: Temporary suspension
- Score < 0.1: Removal from whitelist

## Oracle Node Implementation

### Reference Implementation

```python
import asyncio
from stellar_sdk import Keypair, Server, TransactionBuilder
from datetime import datetime, timedelta

class CeresOracleNode:
    def __init__(self, keypair, contract_id, rpc_url):
        self.keypair = keypair
        self.contract_id = contract_id
        self.server = Server(rpc_url)
        
    async def run(self):
        """Main loop: fetch data and submit every hour"""
        while True:
            try:
                await self.fetch_and_submit()
                await asyncio.sleep(3600)  # 1 hour
            except Exception as e:
                print(f"Error: {e}")
                await asyncio.sleep(300)  # Retry in 5 minutes
    
    async def fetch_and_submit(self):
        """Fetch data for all monitored locations"""
        locations = self.get_monitored_locations()
        
        for geo_cell in locations:
            lat, lon = geohash_decode(geo_cell)
            
            # Fetch rainfall
            rainfall = await self.fetch_rainfall(lat, lon)
            await self.submit_reading(geo_cell, 'Rainfall', rainfall)
            
            # Fetch NDVI
            ndvi = await self.fetch_ndvi(lat, lon)
            await self.submit_reading(geo_cell, 'NDVI', ndvi)
            
            # Fetch soil moisture
            soil = await self.fetch_soil_moisture(lat, lon)
            await self.submit_reading(geo_cell, 'SoilMoisture', soil)
    
    async def submit_reading(self, geo_cell, reading_type, value):
        """Submit reading to oracle contract"""
        timestamp = int(datetime.now().timestamp())
        
        # Sign reading
        message = self.create_message(geo_cell, reading_type, value, timestamp)
        signature = self.keypair.sign(message)
        
        # Submit to contract
        tx = (
            TransactionBuilder(
                source_account=self.server.load_account(self.keypair.public_key),
                network_passphrase=Network.TESTNET_NETWORK_PASSPHRASE,
                base_fee=100
            )
            .append_invoke_contract_function_op(
                contract_id=self.contract_id,
                function_name="submit_reading",
                parameters=[
                    self.keypair.public_key,
                    geo_cell,
                    reading_type,
                    value,
                    timestamp,
                    signature
                ]
            )
            .set_timeout(30)
            .build()
        )
        
        tx.sign(self.keypair)
        response = self.server.submit_transaction(tx)
        print(f"Submitted {reading_type} for {geo_cell}: {value}")
```

## Security Considerations

### Oracle Whitelisting

**Initial Whitelist**:
- Reputable data providers
- Academic institutions
- Community-run nodes

**Addition Process**:
1. Node operator applies with credentials
2. Community review period (7 days)
3. Governance vote (future)
4. Admin adds to whitelist

**Removal Process**:
- Automatic: Reputation score < 0.1
- Manual: Governance vote for misconduct

### Sybil Resistance

**Measures**:
- Whitelisting prevents anonymous nodes
- Reputation system tracks behavior
- Median aggregation limits single-node impact
- Stake requirement (future enhancement)

### Data Source Diversity

**Best Practices**:
- Use multiple data sources per reading type
- Prefer satellite data (harder to manipulate)
- Cross-validate with ground stations
- Monitor for source outages

## Future Enhancements

### 1. Decentralized Oracle Network

**Current**: Admin-controlled whitelist

**Future**: Permissionless oracle network with staking

**Design**:
- Oracles stake tokens to participate
- Slashing for incorrect data
- Rewards for accurate submissions
- Reputation-weighted aggregation

### 2. Advanced Aggregation

**Current**: Simple median

**Future**: Weighted median, outlier detection

**Design**:
```rust
fn weighted_median(readings: Vec<(u32, u32)>) -> u32 {
    // readings: (value, weight)
    let total_weight: u32 = readings.iter().map(|(_, w)| w).sum();
    let target = total_weight / 2;
    
    let mut sorted = readings.clone();
    sorted.sort_by_key(|(v, _)| *v);
    
    let mut cumulative = 0;
    for (value, weight) in sorted {
        cumulative += weight;
        if cumulative >= target {
            return value;
        }
    }
    0
}
```

### 3. Real-Time Data Feeds

**Current**: Daily/weekly updates

**Future**: Real-time streaming data

**Design**:
- WebSocket connections to data sources
- Sub-hourly updates
- Event-driven triggers

### 4. Machine Learning Integration

**Use Cases**:
- Predict crop stress before NDVI drops
- Detect anomalous readings
- Optimize trigger thresholds

**Example**:
```python
def predict_crop_stress(historical_data):
    model = load_model('crop_stress_predictor.h5')
    features = extract_features(historical_data)
    stress_probability = model.predict(features)
    return stress_probability > 0.7
```

---

This oracle specification ensures reliable, tamper-resistant data for parametric insurance triggers while maintaining decentralization and transparency.
