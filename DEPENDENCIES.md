# DEPENDENCIES — plato-vision-jepa

## Signal Chain Layer

**L0 (Sensor Input) — Vision Room Perception**

Camera room perception crate. Produces 16-dimensional vision state vectors from frame diffs, motion vectors, and scene analysis.

## Ecosystem Dependencies

| Repo | Relationship | Description |
|------|-------------|-------------|
| [plato-state](https://github.com/SuperInstance/plato-state) | **Depends on** | Room state vectors that vision state feeds into |
| [plato-nervous](https://github.com/SuperInstance/plato-nervous) | **Depended on by** | Consumes vision state vectors for RoomStateVector fusion and the signal chain |
| [plato-rooms](https://github.com/SuperInstance/plato-rooms) | **Related** | Room definitions where cameras are sensors |
| [plato-tiles](https://github.com/SuperInstance/plato-tiles) | **Related** | Tile types for vision data transport |
| [plato-dashboard](https://github.com/SuperInstance/plato-dashboard) | **Related** | Dashboard renders vision perception status |
| [concrete-token-demo](https://github.com/SuperInstance/concrete-token-demo) | **Related** | Can be exercised through the concrete-token-demo CLI |

## Data Flow

```
IN:
  - Camera frames (raw pixel data or MJPEG stream)
  - Frame metadata (timestamps, resolution)

OUT:
  - 16-dim vision state vector (V₀..V₁₅)
  - Frame diff metrics
  - Motion vector field summary
  - Scene change flags
```
