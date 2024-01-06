A small and easy to use library to generate mazes for games

Irrgarten enables you to procedurally generate mazes of arbitrary size. Compared to similar
libraries, Irrgarten puts special focus on using these mazes within games. The main
advantages are:

### The generated mazes are essentially tilemaps - and walls are just tiles

This is different to other generators that create beautiful mazes but use internal representations
(such as bitmasks) for walls. With Irrgarten you just get a two dimensional vector
like this:

```
1 1 1 1 1 1 1 1 1 1 1 1 1 1 1 
1 0 1 0 0 0 0 0 0 0 0 0 0 0 1 
1 0 1 1 1 0 1 1 1 1 1 1 1 0 1 
1 0 1 0 0 0 1 0 0 0 1 0 0 0 1 
1 0 1 0 1 1 1 0 1 0 1 0 1 1 1 
1 0 0 0 1 0 0 0 1 0 1 0 0 0 1 
1 1 1 1 1 0 1 1 1 0 1 1 1 0 1 
1 0 0 0 1 0 0 0 1 0 1 0 0 0 1 
1 0 1 0 1 1 1 0 1 1 1 0 1 1 1 
1 0 1 0 0 0 0 0 1 0 0 0 1 0 1 
1 0 1 1 1 1 1 1 1 0 1 1 1 0 1 
1 0 0 0 0 0 0 0 1 0 0 0 0 0 1 
1 0 1 1 1 1 1 0 1 1 1 1 1 0 1 
1 0 0 0 0 0 1 0 0 0 0 0 0 0 1 
1 1 1 1 1 1 1 1 1 1 1 1 1 1 1 
```

With walls being ```1``` and the floor being ```0``` you can just 1:1 map this to
any tilemap, gridmap or whatever tile-like graphical structure your favourite engine
offers.

### Injectable randomness

You provide your own random generator. Thus, you are in full control
of the seed and the internal state of the randomness. This enables you to
deterministically generate the same mazes, for example, across the network. Instead
of synchronizing the whole maze, you now only have to synchronize the seed between
the peers. You can also use this seed for replay systems, let testers share difficult
seeds with you to evaluate etc. Invaluable for games with this kind of procedural
content.

# Usage

Add this to your Cargo.toml

```toml
[dependencies]
irrgarten = "0.1"
```

You will also need to choose a random number generator. For the following example,
we simply use the [rand](https://crates.io/crates/rand) crate.

# Example: Simple maze generation

```rust
use irrgarten::Maze;
use rand;

fn main() {
    let mut rng = rand::thread_rng();
    
    // Full maze generation. The dimensions can be any odd number that is > 5.
    let maze = Maze::new(63, 31).unwrap().generate(&mut rng);
    
    // The generated maze data can be accessed via index:
    for y in 0..maze.height {
        for x in 0..maze.width {
            println!("{}", maze[x][y]);
        }
    }
    // Alternatively, Maze also provides into_iter(), iter() and iter_mut()
    // methods to iterate over the columns.
}
```

# Example: Generate an image of the maze (for easy inspection/prototyping)

```rust
use irrgarten::Maze;
use rand;
use std::fs;

fn main() {
    let mut rng = rand::thread_rng();
    let maze = Maze::new(255, 255).unwrap().generate(&mut rng);

    // Save image to disk. Can be opened with most image viewers.
    fs::write("maze.pbm", maze.to_pbm()).unwrap();
}
```

# Example: Use a different random number generator with a seed

Using the [xoshiro](https://crates.io/crates/rand_xoshiro) generator:

```rust
use irrgarten::Maze;
use rand_xoshiro::rand_core::SeedableRng;
use rand_xoshiro::Xoshiro256Plus;

fn main() {
    let mut rng = Xoshiro256Plus::seed_from_u64(123123123);
    let maze = Maze::new(4095, 4095).unwrap().generate(&mut rng);
}
```