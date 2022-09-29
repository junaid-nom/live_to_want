# Live to want

# Simulator revamp

Trying to decide what the core gameplay loop should be and trying to do it in reasonable implementation time.
The big thing to scrap is probably the complex turn based battle system.
Its like a whole game inside this game. And the big issue is that it doesnt synergize with the REAL draw of the game 
which is the simulation system. Instead lets double down on the sim system.

There's youtube videos like these two that show how you can make interesting evolutionary 
simulations with simple code. Lets take that idea and go nuts with it.
https://youtu.be/0Kx4Y9TVMGg
https://youtu.be/N3tRFayqVtk

TODOREVAMP has todos for this revamp

### Revamp combat to be like a normal roguelike. 
You walk around the overworld and can attack stuff next to you.
Items can be used in the overworld. Basic ones to include would be:
 - Wall (blocks path, can be destroyed with attacks)
 - projectile weapon like a bow. infinite ammo, the shot is "slow" so its dodgeable.
 - traps. cause dmg and immobolize for some turn. maybe expensive ones are invisible.
 - Ranks of melee weapons.

### Trait based evolution. 
List of traits that each have a float value and can be increased/decreased per mutation.
 - Some traits have some cost? For example speed requires you to burn more calories.
 - Many traits are only useful in certain circumstances. For example: Tool use. Lets you use certain items.
   - Make it gradual so anything can still craft/use any item but the cost is more and the item is less good (trap does less dmg, immobolize for less time etc)
   - Increasing/decreasing a trait is random and uses up the mutation so that is the downside of these situational traits

### Revamp AI to be evolving.
AI: 
 - List of booleans about what is nearby to the animal. List of objects nearby, and float outputs for the entire list like "Has food somewhere. Has predator somewhere." etc.
 - Output becomes chosing an action, and then its target.
 - action and target is pretty simple either: move toward/away from. attack something. use an item on a target (this is actually alotta options).
 - So we really have 3 ais:
    - 1 that choses which action to do. takes in all items on the list.
    - One that chooses the targets for the action. There would be a separate ai for EVERY action.
      - inputs would always just be a set of floats/bools, but they are run on each item individually and output a preference, highest preference is chosen.
 - Evolution is basically just change the weights between every boolean input and the output.
  

Examples:
U r Deer. Nearby is wolf and some veges.
Goes through list, "predator nearby" input and "run away from target" are heavily weighted so taht wins.
Goes through list to pick target to run away from, chooses wolf as it has the highest result.
-
human sees deer nearby.
chooses action place trap.
puts trap down to the right of the deer.

Questions: 
 - Should personal inventory and stats be part of the input? Yes probably.
    - personal health and food.
    - maybe a bunch of ints for how many of each craftable item can be produced. and how many already exist.
  - How does it relate to evolution? Should some items/behaviors only be unlockable with certain traits? 
    - Actually should make it all gradual.

## TODO LIST for revamp:
 - New AI system. Each creature has an AI component? A list of weights between input->output. simple single layer? I guess I can mess around with this later keep it 1 layer for now.
 - Change attack system.


# Current stuff to do

 - Finish functions login_user_creatures, logout_user_creatures and test them.
 - Make messages that game can process for login/logout.
 - Move user char around stuff

Need to setup the actual game server then do things with the messages.
Most of this should be done in game() in lib.rs

Then start receiving user messages. Need some kind of way to load a player character though that's saved in the mapstate?
Maybe just have a "dormant" creature list in mapstate. Take user characters from there.
If they don't have one make a new user character.

Then when you receive user commands have it do stuff.

Then eventually refactor the battle to take in user input, and also maybe refactor to take in an arbitrary "Battle" interface (so I can put in w/e battle stuff in the future, eh maybe should save that for way later?)

Then actually have different play mode speeds, like do stuff only after user input stuff like that.
Can probably just send an "End Turn" message from the user client to do the next frame?
Or maybe a "Run X Frames" message. Would probably be based on for example movement speed etc or whatever the user sends.
 - Could get tricky because need to have a "how long will this take" for every command?
 - Other way would be make it server based. So server waits until "something" happens to the player then pauses?
Or fuck it if that's too much work just make it continue on like a real server for now (with configurable game speed)?

Sub tasks:
 - Looks like Login/Disconnect messages are NOT forwarded to the game, and are handled in the LoginManager. This is bad because need spawn/dc messages so server adds/removes players.

Long term later on:
 - Maybe eventually use UDP instead of TCP? It would be nice to use a UDP but TCP for some messages thing.


# Systems in place

## Core Loop
Core loop is through lib.rs's run_frame(GameState, GoalNode) method.
GoalNode is how the AI works. The "bones" work for the network graph but haven't
actually built out any AI yet.

### AI
The main idea was to have tree graph. Child nodes can have multiple parents.
Connections are weighted. A parents value is basically the max (or sometimes the sum) of its
child nodes. For example Kill-> Kill Deer-x10> Get Bones. So killing a deer gets 10 bones so the value of kill is same as value of 10 bones.

Each node also has requirements. For example the creature might need to have certain int or traits unlocked to check for 
certain behaviors, for example crafting might require some intelligence or a blueprint unlock or whatever.

 
### Gameplay Systems

## Budding System
Has different soil types. This lets a big bush and grass coexist on the same tile.

After spawning, it has a "blockers" and "unmovable blockers" labeling and it forcefully
moves creatures around so that blockers are the only thing on a tile. This is 

## Battle system
Currently battle system is barely implemented and isn't really done well. 
It also doesn't incorperate player input at all.
But what IS setup is the starting of battle, and the auto-taking of enemy loot on death.
Note though that stuff like bones isn't auto-added to the winners loot. Instead its generated when the creature dies
which is pretty weird cause it lets others steal the loot.


## Networking system
Basically got networking to work with a simple server class that receives
and can send messages through receiever and sender channels. Create one with 
`create_server` in server.rs

You send GameMessages (check messages.rs for all the type).
Messages are wrapped with the username of person sending it.

The server you create handles the getting and receiving of messages.

The server itself handles logging in and disconnects. It will then forward
a LoginMsg or a DropMsg to the receiving pipe, and the game can handle that itself 
(for example create user character or remove the character on dc).


# Tips:

### How file structure works
This is a good summary: https://web.mit.edu/rust-lang_v1.25/arch/amd64_ubuntu1404/share/doc/rust/html/book/first-edition/crates-and-modules.html

Basically you have a lib.rs file which declares all the modules.
`mod mymod;`
All modules either need to be declared:

Right there: 
```rs
mod mymod {
  pub fn blah() {}
}
```

In a file:
Note mymod.rs file name matches the mod mymod; in lib.rs
```
src\
  - lib.rs
  - main.rs
  - mymod.rs
```

In folder with a mod.rs:
```
src\
  - lib.rs
  - main.rs
  - mymod\
    mod.rs
```

This can be recurssive, so mymod\mod.rs could have `mod mysubmodA;` and `mod mysubmodB;`.
Then mysubmodA and mysubmodB must be implemented like above:
- inside mymod\mod.rs 
- new file mysubmodA.rs
- new folder mymod\mysubmodA\mod.rs

### Tests
For a library crate you will need to import:
extern crate mycrate;
use mycrate::mymod::blah;


### Compile and run
`cargo run` 
Does nothing really right now but good for checking compiler.
Note runs main.rs. A crate can have both a lib.rs and a main.rs

`rustup update`
updates rust version

### Run tests
`cargo test`
Runs all tests. No captured output.

`cargo test -- --show-output`
Runs all test, and shows output

`cargo test test_name -- --show-output`
Runs tests that have matching name filter

`cargo test --color always`
to display colors

`cargo test --release --test-threads=17`
To test with optimizations on

`$env:RUST_BACKTRACE=1`
Turn on backtrace for powershell

### dependencies weirdness:
the deep_ai requires libtorch which you gotta do some weird stuff for:
https://crates.io/crates/tch
See "Libtorch Manual Install" which talks about how to get it working on windows

You will need GCC to get it to work which you can get by following this:
https://code.visualstudio.com/docs/cpp/config-mingw
 - install mysys2 https://www.msys2.org/
 - run `pacman -S --needed base-devel mingw-w64-x86_64-toolchain` in a mysys2 terminal
 - for PATH env var add path to the bin something like: `C:\msys64\mingw64\bin`

Reopen powershell for changes to take affect ofc.
C:\Users\xjuna\Documents\live_to_want\deep_ai