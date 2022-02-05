# Live to want

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

