# Copilot Instructions for chess-rust

## Project Overview

This is a multiplayer web chess game backend implementation built in Rust using the Axum web framework. The server provides a REST API that enables players to create games, join games, and make moves from different devices.

## Required Before Each Commit

- Run `cargo fmt` before committing any changes to ensure proper code formatting
- Run `cargo test` to ensure all tests pass
- Run `cargo clippy` to check for common mistakes and improve code quality
- Ensure `cargo build` completes without errors

## Development Flow

- **Build**: `cargo build` - Compile the project
- **Run**: `cargo run` - Start the server on `http://0.0.0.0:3000`
- **Test**: `cargo test` - Run all unit tests (30 tests should pass)
- **Format**: `cargo fmt` - Format code according to Rust style guidelines
- **Lint**: `cargo clippy` - Run linter to catch common mistakes

## Repository Structure

```
src/
├── main.rs           # Application entry point, server setup
├── api.rs            # REST API handlers and routing
├── chess/
│   ├── mod.rs        # Chess module exports
│   ├── piece.rs      # Piece types (Pawn, Knight, Bishop, Rook, Queen, King) and colors
│   ├── board.rs      # 8x8 board representation and manipulation
│   └── game.rs       # Game rules, move validation, check/checkmate detection
└── game/
    ├── mod.rs        # Game module exports
    ├── session.rs    # Game session and player management
    └── state.rs      # Shared state for multiple concurrent games
```

## Architecture

The project is organized into three main modules:

- **chess**: Core chess game logic
  - `piece.rs`: Piece types (Pawn, Knight, Bishop, Rook, Queen, King) and colors (White, Black)
  - `board.rs`: 8x8 board representation and manipulation
  - `game.rs`: Game rules, move validation, and game state management
- **game**: Multiplayer session management
  - `session.rs`: Game session and player management
  - `state.rs`: Shared state for multiple concurrent games
- **api**: REST API handlers and routing using Axum

## Coding Standards

### General Rust Conventions

- Follow standard Rust naming conventions (snake_case for variables/functions, PascalCase for types)
- Use `cargo fmt` to format code before committing
- Address all compiler warnings - run `cargo fix` when appropriate
- Prefer explicit error handling with `Result<T, E>` over panics
- Use descriptive variable names that convey intent

### Code Style

- Keep functions focused and reasonably sized
- Use pattern matching (`match`) for control flow when appropriate
- Leverage Rust's type system for correctness (enums, structs with meaningful fields)
- Prefer composition over inheritance using traits
- Use `derive` macros for common trait implementations (Debug, Clone, Serialize, Deserialize)

### Error Handling

- Return `Result<(), String>` for operations that can fail, with descriptive error messages
- Use `ok_or_else()` for Option to Result conversions with meaningful error messages
- Error messages should be user-friendly and descriptive

## Testing

### Test Structure

- Tests should be placed in a `tests` module at the bottom of the file using `#[cfg(test)]`
- Use descriptive test function names that explain what is being tested (e.g., `test_pawn_forward_move`, `test_invalid_move_wrong_turn`)
- Each test should focus on a single behavior or scenario

### Test Coverage

- All new game logic features must include unit tests
- Test both success and failure cases
- Test edge cases and boundary conditions (e.g., out of bounds, invalid coordinates)
- Run `cargo test` before committing changes

## Key Guidelines

1. Follow Rust best practices and idiomatic patterns
2. Maintain existing code structure and organization
3. Write unit tests for new functionality using `#[cfg(test)]` modules
4. Use descriptive test names that explain what is being tested
5. Document complex logic with clear comments
6. Validate all user input before processing
7. Use proper error handling with `Result<T, E>` types

## Dependencies

- **axum**: Web framework for REST API
- **tokio**: Async runtime
- **serde**: Serialization/deserialization
- **uuid**: Unique game identifiers
- **tower-http**: Middleware (CORS, tracing)
- **tracing**: Logging and diagnostics

Only add new dependencies when absolutely necessary. Prefer using existing crates already in the project.

## Chess Game Rules

### Coordinate System

The board uses a 0-7 coordinate system:
- Row 0 = White's back rank (a1-h1)
- Row 7 = Black's back rank (a8-h8)
- Col 0-7 = Files a-h

### Move Validation

- Validate piece ownership (player must own the piece being moved)
- Validate turn order (correct player's turn)
- Validate move is legal for the specific piece type
- Validate destination doesn't contain friendly piece
- For sliding pieces (Bishop, Rook, Queen), validate path is clear

## API Design

- Use appropriate HTTP methods (GET for reads, POST for writes)
- Return JSON responses with proper status codes
- Include descriptive error messages in error responses
- Maintain RESTful design principles
- Enable CORS for web frontend integration

## Security Considerations

- Validate all user input (coordinates, player IDs, game IDs)
- Check array bounds before accessing board positions
- Validate game state before allowing operations
- Prevent players from making moves out of turn
- Prevent players from moving opponent's pieces

## Code Review Checklist

Before submitting code:
- [ ] Code compiles without errors
- [ ] All tests pass (`cargo test`)
- [ ] Code is formatted (`cargo fmt`)
- [ ] No new compiler warnings
- [ ] Appropriate tests are included for new functionality
- [ ] Error handling is proper and descriptive
- [ ] API changes are documented in README.md if applicable
