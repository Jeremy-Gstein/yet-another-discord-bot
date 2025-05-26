# Yet another Discord bot (BDAY)

## Bday is a expiriment working with serenity and poise discord api framwork.

### Purpose
- test and develop features for [sg_assist](https://github.com/Jeremy-Gstein/sg_assist) (yet another... discord bot)
- expose your local shell for the whole chat! (see [Disclaimer](#Disclaimer) ) 
### Disclaimer:
- This software by design wraps around the hosts 'shell' 
- At a minimum please consider building in a isolated enviroment (like docker) 
- However there are known ways to 'escape' sandboxed enviroments

<mark> USE AND BUILD AT OWN RISK! THIS IS YOUR ONLY WARNING!!</mark>

---
### Build
- Requires docker for _safer_ runtime enviroment. 
- clone repo, cd repo, and create a .env file.
```SHELL
git clone https://github.com/Jeremy-Gstein/yet-another-discord-bot
cd yet-another-discord-bot
touch .env
```
- .env need the following env vars to build (a cloudflare r2 bucket is needed for /mp3)
```SHELL
cat << "EOF" > .env
DISCORD_TOKEN=
ACCESS_KEY=
SECRET_ACCESS_KEY=
ENDPOINT_URL=
PUBLIC_DOMAIN=
BUCKET_NAME=
EOF 

```
- build the dockerfile image and tag it as bday
```SHELL
docker build -t bday .
```
- run bday:latest image after building locally. 
```SHELL
docker run -itd --env-file .env --name bday_app bday:latest
```
