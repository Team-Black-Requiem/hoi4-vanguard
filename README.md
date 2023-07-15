# hoi4-vanguard
A HOI4 script toolset  
Requires Python


## GFX-Walk
GFX-semifinal usage instructions: drop in your root mod folder, and run python primary.py
Goes for a walk() through specified folders to find relevant files and make .gfx entries for them.

Arguments

These will only search for .dds files.
--goals-shines --ideas --event-pictures --leader-gfx

--subfolder can specify any given /gfx/ folder like so:  --subfolder=interface/scripted_gui_graphics  
--subfolder can also specify a full filepath if you're feeling a bit deranged --subfolder="C:\Users\<User>\Documents\Paradox Interactive\Hearts of Iron IV\mod\Test_mod\gfx\interface\scripted_gui_graphics"  
--subfolder will handle .png files as well as .dds  
--subfolder will generate shines as well if you point it at interface/goals instead of using --goals-shines  

