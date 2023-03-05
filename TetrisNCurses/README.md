# Terminal Tetris
Simple text UI tetris game for the terminal - ported to rust from [tinytetris](https://github.com/taylorconor/tinytetris).
UI based on the NCurses TUI library.
Similar size (source code) as tinytetris, but uses less memory - tetromino shapes stored more compactly.

```
% cargo run
```

Controls: 
* q to quit
* Arrow Left & Arrow Right to move sideways
* Arrow Up to rotate
* Arrow Down to drop
