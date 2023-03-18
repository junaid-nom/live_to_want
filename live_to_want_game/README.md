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
 - Looks like Login/Disconnect messages are NOT forwarded to the game, and are handled in the LoginManager. This is bad because need spawn/dc messages so server adds/removes players. (Is this actually true because later on in the networking section it says server will forward those messages?)

Long term later on:
 - Maybe eventually use UDP instead of TCP? It would be nice to use a UDP but TCP for some messages thing.

# TODONEXT:
Rock paper scissor style plants, THE NETWORKING FIRST SO ITS READY TO MAKE UNREAL CLIENT then animals, then ai for the animals, then combat.

###  Rock Paper Scissor Plants, Animals, Crafting, Combat:
3 soil types. And 3 heights a plant can be.
A given space can have 1 soil type, and have 2 plants of different heights (low, high) (low, med) (med, high).
Each plant can actually live in 2 different soils. But it will spread a soil type.
So a plant could be in soil type A, but the area around it would become soil type B.
This should have interesting combinatorics.
Height and soil type determine what items the plants drop when killed.

Animals meanwhile have traits for ability to digest each plant height and each plant soil.
So 6 in total? Or maybe the traits are like you can eat 2/3 of the heights/soils (still 3 of each tho)
Maybe also have a trait for EATING animals based on their diet. So one animal could get
a bonus for eating animals that eat low+SoilA. There would be 9 of these carnivore traits?
These digestion traits also influence what items they drop when killed.

Crafting requires items from plants and animals to make. So you could for example
sabotage a civilization but spreading an animal that eats plants of a certain soil
or predators that eat animals that eat plants that the civ doesn't want.
Basically messing with ecosystem can have a big influence on what items are easily craftable.

Combat itself should be roguelike CDDA style. This is because there has to be "overworld"
mechanics anyway necessary to START the fight (run speed etc) so fuck it might as well 
just focus on that. Also it can be relatively simple but have super interesting AI
interactions for example melee tanks defending long range immobile archers.
COmbat also should be rock paper scissorsish.
3 damage types and 3 armor types. Thus you can cripple a civ by removing ingredients
necessary to build armor for the damage type you invested in making.
Additionally have tradeoff between:
 - Range, Attack Speed, Mobility.
So each weapon should be good with 2 of the above and bad with 1 or other tradeoff combos.
For example long range, high attack speed, but you can't move while shooting at all?
Vs Low range(melee) but very fast and u can move while attacking.
Or VERY high attack speed but immobile, so equiping with high def stuff could help? or something.

# TODO Eventually:

## AI upgrades
New NodeListTarget::ItemTarget(ItemType). This can then be used to make nodes like "PickUpItem" for all items(nearby/in inventory). And it would dynamically
set the effect based on the item.

Can also probably use it for "UseItem" and it would have dynamic requirements. And the reward would be a big match on all item types, which is probably good cause then reminds devs whenever a new item is added, to figure out its reward function.




## Integrate with a Client
Lets use unreal so I can put unreal on resume and because it looks better out of the box.
C++ libraries might also help? Or maybe it will be awful.
The way communication would work with client:
 - Server sends entire gamestate of region every frame (lets say .1 seconds).
   - However, for each creatureState it doesn't give all the details, just the core like location etc.
   - Client can click on a creature to "inspect" and get more details. Sends out a message to the server which replies with the details next frame.
 - Client has a list of spawned creatures and spawned tiles. When it gets the server message, for each of these gameobjects it will see if their is a spawned version of them already. if there is, it'll update them based on the info in the message. If there isn't it'll make a new gameobject (pooling?).
   - Death: First step of each message processed is to go through all the spawned gameobjects and mark them as "invalid", then when it updates them all it will mark them as valid. Any gameobject that is still invalid after processing the message is destroyed.

Alternatives: 
 - DS combat game multiverse craziness. AI enemies. Dark atmosphere and particle effects.
 - Lawnchairman: Telegod. Game where you refuse to get off chair, teleport with balls.
   - Start out like a lefty believing everything. Taking pills that are a metaphor for trans.
   - Obese uneducated working class losers are only ones who don't agree. Say that "The rich and greedy bribe them".
   - Face office-chair people. Man bad guy pays outsource people to copy MC's tech. THey also refuse to get out of chair because they lack creativity to do anything other than make an exact copy.
   - Other jokes.

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

You send GameMessages (check messages.rs for all the types).
Messages are wrapped with the username of person sending it.

The server you create handles the getting and receiving of messages.


There is also ConnectionManager server which handles logging in and disconnects. It will then forward
a LoginMsg or a DropMsg to the receiving pipe, and the game can handle that itself 
(for example create user character or remove the character on dc).

The way the ConnectionManager works is, client first sends a login message.
ConnectionManager handles logging in and auth stuff with password.
If login is successful it adds the client to the valid login conection list.
Then the server can send messages via `send_message` to a particular user and `send_message_all`
to all users.


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

`cargo test --no-fail-fast`
Runs all tests, even if one fails it'll continue.

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

### Regex examples:

Find: `Gary([^A])`
Replace: `GaryAllen$1`
The `[^A]` means any char not A.
any `()` "groups" it and then in replace you can put $1, $2, etc.
So the ([^A]) in paranthesis so you can replace it later with the $1 to put it back.

