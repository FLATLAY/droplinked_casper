# Usage Testing of [droplinked](https://www.droplinked.com) contract

** Note : We are about to do a major release so this document would get updated as the system is updated

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
### 2. Create an account on droplinked & Record your Product

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

Hit the `New Product` Button to create a new product. Fill in the needed information as I do : 

![image](https://github.com/FLATLAY/droplinked_casper/assets/20683538/38b1844a-aa74-4dbf-9f68-3c27300a8778)

![image](https://github.com/FLATLAY/droplinked_casper/assets/20683538/acb00757-f078-444f-8928-a6d3a4f15070)

![image](https://github.com/FLATLAY/droplinked_casper/assets/20683538/af157e44-f08d-4e34-a10c-4de1a43400d5)

Add variants to your prodcut and hit save on each of them 

![image](https://github.com/FLATLAY/droplinked_casper/assets/20683538/8c9d92c1-b145-4aa5-9e4b-f3cb2fb424f1)

![image](https://github.com/FLATLAY/droplinked_casper/assets/20683538/dc4aec5d-373a-40b5-afd2-a9e9d447f465)

And push the `Publish` button to create your product. 

![image](https://github.com/FLATLAY/droplinked_casper/assets/20683538/cd95ec4c-2183-4844-a463-679b62db7e50)

From the above page, select edit product (on the right hand of the product) 

![image](https://github.com/FLATLAY/droplinked_casper/assets/20683538/3bc55382-bad1-4e66-9639-71b743f2a088)

Open up your wallet, move to `Producer` account, and hit `Connect` button to connect to droplinked, And then
In the product variants section, click on the green icon (Record product)

![image](https://github.com/FLATLAY/droplinked_casper/assets/20683538/0a546635-cb1a-4d01-8d5a-f2c337dc0035)

![image](https://github.com/FLATLAY/droplinked_casper/assets/20683538/58cc52a9-f940-4e9f-82ce-b5b3521f1256)

Choose the BlockchainNetwork to casper, and enter a comisson (between 1 and 100) and hit the `Drop` button 

your wallet would be opened and ask you to sign a text, click on `sign`

![image](https://github.com/FLATLAY/droplinked_casper/assets/20683538/91269eb3-cfe9-4a1e-8862-07a64f1f0923)

Then another wallet pop up would be open to ask you verify the deploy, hit sign on that too

![image](https://github.com/FLATLAY/droplinked_casper/assets/20683538/e13d0a69-eaf2-44f8-80aa-dc76add3a8c3)

A popup window would open to you which shows you the deploy hash and the link to testnet, click on it, you would go to the deployment page on testnet, and after a minute or two, the deploy would execute succssfully 

The deploy hash that was shown in this example was : https://testnet.cspr.live/deploy/004ba727d6175c274eff53177998579b0c35c35cab848f01a11c12b035ba6ef1

![image](https://github.com/FLATLAY/droplinked_casper/assets/20683538/16718e0d-f950-4173-9676-cf4cd539f324)
