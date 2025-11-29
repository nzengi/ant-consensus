# AntColony Consensus

Blockchain-less distributed consensus system inspired by ant colony optimization algorithms.

## Overview

AntColony Consensus is a novel distributed consensus protocol that uses digital pheromones and mobile ant agents to achieve agreement across a network of nodes without requiring a blockchain. The system is inspired by how real ant colonies use pheromone trails to find optimal paths and coordinate behavior.

## Features

- **Pheromone-based Communication**: Nodes emit digital pheromones to signal consensus values
- **Ant Agent System**: Mobile agents explore the network and spread information
- **Self-Organizing**: No central authority required
- **Fault Tolerant**: Natural resilience through pheromone evaporation
- **Efficient**: Low overhead compared to blockchain systems

## Architecture

### Core Components

1. **Pheromone System**: Digital trails that represent consensus values
2. **Ant Agents**: Mobile entities that carry and spread pheromones
3. **Node State**: Local state management for each node
4. **Network Layer**: UDP multicast for communication
5. **Consensus Engine**: Main algorithm coordinator

## Building

```bash
cargo build --release
```

## Running

```bash
# Start a node with default settings
cargo run -- --node-id 1

# Start with custom multicast address
cargo run -- --node-id 2 --multicast-addr 239.255.0.1:5000 --port 5001

# Enable verbose logging
cargo run -- --node-id 3 --verbose
```

## Testing

```bash
# Run unit tests
cargo test

# Run integration tests
cargo test --test integration_tests
```

## How It Works

1. **Proposal Phase**: A node proposes a consensus value by emitting a pheromone
2. **Exploration Phase**: Ant agents are created to explore the network
3. **Propagation Phase**: Ants follow pheromone trails, strengthening popular paths
4. **Evaporation Phase**: Weak pheromones evaporate over time
5. **Consensus Phase**: When pheromone intensity reaches threshold, consensus is reached

## Configuration

Key parameters can be adjusted in the source code:

- `CONSENSUS_THRESHOLD`: Pheromone intensity required for consensus (default: 0.8)
- `DEFAULT_EVAPORATION_RATE`: Rate at which pheromones weaken (default: 0.01)
- `INITIAL_ANT_ENERGY`: Starting energy for ant agents (default: 100.0)
- `ENERGY_DECAY_RATE`: How fast ants lose energy (default: 0.1)

## License

MIT

