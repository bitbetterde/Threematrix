# Threematrix
![Mastodon Follow](https://img.shields.io/mastodon/follow/107838426834517530?domain=https%3A%2F%2Fmastodon.social&style=social) 
![Matrix](https://img.shields.io/matrix/threematrix:matrix.org?label=Chat%20on%20Matrix&style=social)
> Threematrix is a work-in-progress bridge software – written in Rust – between the Threema messenger (based on the Threema Gateway API) and the Matrix protocol. It's currently being funded via the German Prototype Fund.

## Status
This project was submitted and accepted for funding via the 11th round of the [German Prototype Fund](https://prototypefund.de/), which aims to fund public interest technologies under open source licenses. The funding period started on **March 1st 2022** and will last for six months until the end of August 2022. The software is currently in Alpha stadium (meaning it can be tested, but shouldn't be used in production, yet).

## Cost Disclaimer
Currently this software is built to work with Threemas commercial Gateway API product – which uses a [per-message pricing model](https://gateway.threema.ch/en/products). Their **pricing model is not well suited for the use with a messsenger bridge**, because typically controlling the amount of messages is out of the hand of the bridge admin. Also, group messages are just implemented as regular text messages to every group member. So for the bridge to receive a single message from Matrix and forward it to a 100 member Threema group, will be equal to sending 100 single text messages (costing you 100 credits or roundabout 1 Euro). **BE AWARE THAT THE BRIDGE MIGHT USE UP YOUR THREEMA CREDITS VERY FAST – DON'T PUT LARGE AMOUNTS OF CREDITS/MONEY IN YOUR ACCOUNT AND BE CAREFUL WHEN ADDING THE BRIDGE TO LARGE THREEMA GROUPS.**

We have been in contact with Threema and have told them how unfortunate the current pricing model is for use in a messenger bridge. They seemed to understand our problem and they will think about possible solutions for this problem. We hope that we will be able to find a (more affordable) pricing solution for future use of our bridge.

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

