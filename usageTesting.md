# Usage Testing of [droplinked](https://www.droplinked.com) contract

This repository contains the logic and code for the droplinked's contract, below is a short introduction to droplinked : 

On the droplinked protocol, we are registering products on chain and to enable 3rd party publishers to leverage these registered products and sell them across any marketplace, dapp or native site in order to earn commission. We are complimenting this with headless tooling for NFT Gated store fronts on droplinked.com and other valued added NFT solutions. This particular repository contains the customized contract for the Casper Network.


## The Process of testing the contract (and droplinked's system) is as follows : 

___
### 1. Create 3 accounts on your casper wallet and fund them using casper's testnet faucet

You'll need 3 accounts to test the contract, you can name them `producer`, `publisher` and `customer` as we will use these names in this documentation.
You should create these accounts on your `casper wallet` and faucet them on testnet, the process is as follows : 

Unlock the casper wallet first 

![image](https://github.com/FLATLAY/droplinked_casper/assets/20683538/c5e052f5-464c-4d49-9745-46c1012a9197)

Select the settings section and hit the Create Account button there

![image](https://github.com/FLATLAY/droplinked_casper/assets/20683538/7ef1cbd4-ce52-43a8-a21d-1dcdfe8dd659)

Give a name to your account and hit create account

![image](https://github.com/FLATLAY/droplinked_casper/assets/20683538/44145c4f-dc97-4e43-a7ab-7936db9fb5c1)

You should do this for all the 3 mentioned accounts. Then you'll need to faucet your accounts on testnet, Select your account on your wallet and hit `Manage Account`

![image](https://github.com/FLATLAY/droplinked_casper/assets/20683538/6cbcdb09-2e1f-4213-ae6a-00406eb45e86)

Hit view on CSPR.live

![image](https://github.com/FLATLAY/droplinked_casper/assets/20683538/2be0a0c2-c4fb-4b1d-80f0-3f1bd414f580)

When you are on casper testnet, hit the connect button on your account

![image](https://github.com/FLATLAY/droplinked_casper/assets/20683538/298bf314-7f9d-48b1-bd05-a1bebc00e835)

Then, from the tools menu, hit the faucet button, 

![image](https://github.com/FLATLAY/droplinked_casper/assets/20683538/d7f8ddef-19c4-4fa7-b362-cce736b4f9c3)

Fill in the captcha, and hit the `Request Tokens` button, It would take up to 2 minutes to give you 2000 CSPRs. Do as so for other 2 accounts, So at the end you would have 3 accounts with 2000 CSPRs in each of them.

___
### 2. Create an account on droplinked

Move to https://ngsf.flatlay.io/, Enter your username and hit the sign up button

![image](https://github.com/FLATLAY/droplinked_casper/assets/20683538/e2868478-625e-40d1-980e-9cf85aba256b)

Fill the requested information and hit sign up

![image](https://github.com/FLATLAY/droplinked_casper/assets/20683538/df8ff767-b042-4e7b-bbcb-81b26cb2dd66)

Then you have to verify the link sent to your email 

![image](https://github.com/FLATLAY/droplinked_casper/assets/20683538/a34ef1db-8b82-4a1e-a068-17e92f80d515)

Hit the Login button and fill in your data 

![image](https://github.com/FLATLAY/droplinked_casper/assets/20683538/8f31ebba-e0b6-4170-9ef2-9a65d3a7e4b1)

And log into your account

You should fill in the requested data 

![image](https://github.com/FLATLAY/droplinked_casper/assets/20683538/01f2c9d0-dd4d-4dfc-ba1a-057c9814259a)

** In the address part, make sure you enter a valid address (for easypost to access), A valid address is brought to you in the next pic

![image](https://github.com/FLATLAY/droplinked_casper/assets/20683538/442ecd9d-4ff5-4fea-8c3f-315ebea44e1e)

Hit save, And fill the store design part as you desire, Then in the `payment options` page, choose the IMS type to droplinked, 

![image](https://github.com/FLATLAY/droplinked_casper/assets/20683538/166626f9-fbcb-42c2-88c3-d4b0e4403d4c)

Activate the casper payment and paste the publickey of the `producer` here and hit Next.

![image](https://github.com/FLATLAY/droplinked_casper/assets/20683538/736f356b-b23e-493f-9666-fe2e162bf239)

Fill in the next page (contact page) as you desire and hit the `Publish Store` button.

You would be transferred to the `Products` page of droplinked.

![image](https://github.com/FLATLAY/droplinked_casper/assets/20683538/1c226eef-2a34-41c7-9877-408a46218f28)


