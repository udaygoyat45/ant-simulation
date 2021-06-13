# ant-simulation

An Ant Simulation inspired by https://www.youtube.com/watch?v=X-iSQQgOd1A. 
I found the particle simulation in the video really awesome, and someone also recommnended me to learn Rust. 
I first implemented this in python, but unfortunately, PyGame couldn't handle that many ants.
I used GGEZ and implemented all the physics and collision detection myself. I attempted to use rayon for parallelization. 
I just realized that one can use a preexisting particle system and shaders to handle a lot more ants, so if someone would be interested in doing so, feel free to contribute!

The main constants can be found in `src/main.rs`. You can tweak them to see how the game changes. 
You can also find the initializations of the ant struct in `src/ant.rs`, and you can also find a few constants there. 
