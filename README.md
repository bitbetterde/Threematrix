# Threematrix
![Mastodon Follow](https://img.shields.io/mastodon/follow/107838426834517530?domain=https%3A%2F%2Fmastodon.social&style=social) 
![Matrix](https://img.shields.io/matrix/threematrix:matrix.org?label=Chat%20on%20Matrix&style=social)
> Threematrix is a work-in-progress bridge software – written in Rust – between the Threema messenger (based on the Threema Gateway API) and the Matrix protocol. It's currently being funded via the German Prototype Fund.

## Status
This project was submitted and accepted for funding via the 11th round of the [German Prototype Fund](https://prototypefund.de/), which aims to fund public interest technologies under open source licenses. The funding period started on **March 1st 2022** and will last for six months until the end of August 2022. The software is currently in Alpha stadium (meaning it can be tested, but shouldn't be used in production, yet).

## Cost Disclaimer
Currently this software is built to work with Threemas commercial Gateway API product – which uses a [per-message pricing model](https://gateway.threema.ch/en/products) and has a one-time setup fee of around 64 Euro. Their **pricing model is not well suited for the use with a messsenger bridge**, because typically controlling the amount of messages is out of the hand of the bridge admin. Also, group messages are just implemented as regular text messages to every group member. So for the bridge to receive a single message from Matrix and forward it to a 100 member Threema group, will be equal to sending 100 single text messages (costing you 100 credits which equals a price between 2 and 5 Euro – depending on how much credits you bought). **BE AWARE THAT THE BRIDGE MIGHT USE UP YOUR THREEMA CREDITS VERY FAST – DON'T PUT LARGE AMOUNTS OF CREDITS/MONEY IN YOUR ACCOUNT AND BE VERY CAREFUL WHEN ADDING THE BRIDGE TO LARGE THREEMA GROUPS.**

We have been in contact with Threema and have told them how unfortunate the current pricing model is for use in a messenger bridge – especially for private/hobbyist use. They seemed to understand our problem and they will think about possible solutions for this problem. We hope that we will be able to find a (more affordable) pricing solution for future use of our bridge.

## Security Disclaimer
Both Threema and Matrix are products known for their E2E Encryption capabilities. While "both sides" might offer strong encryption, a messanger bridge is conceptually always a weak point in encryption. To forward messages the bridge needs to decrypt the incoming message and encrypt it again for the outgoing side. This means that the bridge is capable of reading the content of messages passing through it. **Please don't use a bridge for sensitive communication and make sure you know who has access to the bridge server.**

## Setup instructions


### Getting a Threema Gateway Account and Threema ID
Sign up for a [Threema Gateway account](https://gateway.threema.ch/en/signup), charge it up with at least 1600 credits (the setup fee to get a Threema ID) and request a new "End-to-End" Threema ID in the backend (enter your desired ID and username and generate a key pair according to their instructions – you can leave the URL empty for now).

### Create Matrix bot account on your Matrix Homeserver
Sign up for a new user account on your favorite Matrix homeserver, e.g. via Element. Take note of your username and password, you will need to add it to the `threematrix_config.toml` file.

### Set up reverse proxy
Threema calls a `/callback` HTTPS endpoint on your server, every time you receive a message. So you need to make sure that your server can be reached from the internet via a domain name (IP address will not work, because it needs to have a TLS certificate – which is only available for domain names, not for IP addresses). You need to set up a reverse proxy to accept and terminate TLS connections. As an example, you could use [Caddy](https://caddyserver.com/) as a reverse proxy with the following command:

```
caddy reverse-proxy --from mydomain.com --to localhost:8888
```

### Clone repo and build project
Clone the repository to your server and install rust (we recommend using [rustup](https://rustup.rs/)), then build the binary via `cargo build --release`

### Edit config file
Add Threema Gateway data (`secret`, `private_key`, `gateway_own_id`) and Matrix config (`homeserver_url`, `user`, `password`) to the config file. See the `threematrix_cfg_example.toml` for example data.

### Run the binary
From your root folder (the folder where you cloned the repo), run `./target/release/threematrix` and hopefully you should see output like this:

```
   Compiling threematrix v0.1.0 (/Users/myself/Threematrix)
    Finished dev [unoptimized + debuginfo] target(s) in 8.88s
     Running `target/debug/threematrix`
INFO [threematrix] Starting Threematrix Server v0.1.0. Waiting for Threema callback on localhost:8888
```

### Invite Bot to the Rooms
Now you can invite the Threema user to your Threema group, and also invite the bot user to your desired Matrix room. **Also, you need to give the bot user moderator rights (power level >= 50).**

## Bind rooms
Send `!threematrix bind !a1b2c3:myserver.com` via Threema to bind two rooms together. It is not necessary to rebind after the bridge has crashed or restarted, but it is required to send a Message from the Threema side first. If you don't do this, Matrix messages might get lost – even though the bridge is running.

## Motivation
While Threema is a great messenger app for many purposes, it can become difficult to use for larger organizations. The lack of room directories or the limitation of groups only having a single admin user are hard to work around once your organization grows bigger. For users it's very hard to leave Threema behind, even though theoretically it is an Open Source project, because in reality there are very few 3rd-party-integrations of the Threema protocol. We're trying to open Threema up to the world of Matrix.

## Team
We are Fabian and Moritz, two software developers from Hamburg, Germany. We initially met during our Computer Science bachelor programme at HAW Hamburg and have been developing various freelance software projects under our brand name **bitbetter** since then.

## Community
Feel free to join our Matrix room [#threema-bridge:matrix.org](https://matrix.to/#/#threema-bridge:matrix.org) if you want to follow our development process. Also, you can follow our [Mastodon-Account @threematrix@mastodon.social](https://mastodon.social/web/@threematrix) to stay up to date.

## Legal Disclaimer
This project has no connection or affiliation with any of the involved "sides" – neither Threema, Threema GmbH nor Matrix.

## Funding
<div style="display: flex;">
<a href="https://www.bmbf.de/"><img src="https://user-images.githubusercontent.com/4677417/159274561-ca7a0f0f-b7cf-4a91-a6bc-b38e1f768b11.svg" width="250px" /></a>
<a href="https://prototypefund.de/project/threematrix-eine-bruecke-zwischen-threema-und-dem-matrix-protokoll/"><img src="https://user-images.githubusercontent.com/4677417/159274772-bd4a0ea2-ef2e-4578-89fe-f87e61d21e73.svg" width="250px" /></a>
</div>

