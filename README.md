# Yet another Discord bot (BDAY)

## Bday is a expiriment working with serenity and poise discord api framwork.

### Purpose??
	- test and develop features for sg_assist (yet another... discord bot)
	- expose your local shell for the whole chat! -- PLEASE DONT DO THIS! USE AND BUILD AT OWN RISK! THIS IS YOUR ONLY WARNING!!
### Build
	- clone repo, cd repo.. note dockerfile, it uses cargo watch and mounts the local volume to the container allowing for hot-reload development.. 
		* What do i mean?? - you clone repo and cd repo and work in that repo. each write to a file in /usr/src/crabby will force cargo watch to do a fresh build with LSP/lint as usual.
	- ``docker build -t bday``
	- ``docker run -itd --rm -v $PWD:/usr/src/crabby -w /usr/src/crabby -e DISCORD_TOKEN=MyCoolToken bday




### UNSAFE?!?!?
	- yeah i wanted to test how much host os information in leaked to the container through shell/posix api and quickly noticed that DISCORD_TOKEN=123asd was plain and readable with a quick `printenv`..
	- this will be worked out at some point, perhaps adding another container layer between app executable env.

