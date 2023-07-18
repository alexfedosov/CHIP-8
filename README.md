# CHIP-8 Emulator

![CHIP-8](https://github.com/alexfedosov/CHIP-8/assets/919187/24032844-fa0f-40fc-9e96-54670b488558)


A few days ago, I once again stumbled across a mention of a fantasy console. I've always been curious about fantasy consoles, like PICO-8, and how they work (also have been thinking about building my own as an exploratory project). 

> *You can find many fantasy console implementations here* 
> https://github.com/topics/fantasy-console

> *Here is a substantial collection of fantasy consoles as well*
> [FANTASY CONSOLES/COMPUTERS](https://paladin-t.github.io/fantasy/)

It turns out that most of them embed a ready-made VM (with LUA being a popular choice) to run ROM's code and provide a set of APIs to interact with virtual hardware. While this approach is very practical, I wanted to delve a bit deeper, taking a step one level down, which led me to the fascinating world of emulators. 

The internet suggests that people typically implement a CHIP-8 emulator to get their foot in the door since it's fairly simple and only has around 35 instructions. This seemed like a manageable project for a weekend. 

I've decided to go with Rust, because I wanted to approach it one more time and also it does look like a perfect fit for the role, being high-level but GC free at the same time (don't look at me C++). Since both my knowledge of Rust is pretty basic and I had no idea about internals of emulators, I pretty much followed a fantastic [open-source book](https://github.com/aquova/chip8-book) on that topic.

> *This is also an excellent page to start with CHIP-8*
> [Awesome CHIP-8](https://chip-8.github.io/links/)

I managed to mess up setting `v_regs[0xF]` to the correct value when addition overflows. This easy-to-overlook mistake cost me a few extra hours of debugging.


## How to run it:

- Install SDL library: https://crates.io/crates/sdl2
- Download some games: https://johnearnest.github.io/chip8Archive/
	+ Make sure the game is for original chip8, not super chip8 or other modifications
- `cargo run <game>`
