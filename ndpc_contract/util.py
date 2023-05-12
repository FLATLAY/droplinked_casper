import subprocess
import json
import requests
from termcolor import colored

rpc_node = "http://195.201.174.222:7777"
contract_hash = "7f2fb764e619c0143040becc8b46e1861e7b72d31f4e55cea249435fee05eb80"

acc_1_name = "acc1"
acc_1_hash = "95dcc50ad49351dd97aab4678d9926de18981ab611903c145341df252300e88a"
acc_1_secret = "keys/acc1_secret_key.pem"

acc_2_name = "acc2"
acc_2_hash = "8e11ebbcdf47ecc0fcdf190882a1c25502a4ee17b1ebf49df0c076478bbcaf7d"
acc_2_secret = "keys/acc2_secret_key.pem"

acc_3_name = "acc3"
acc_3_hash = "87b67c360aa7d4718d0a47caa2c9653d0bebe35fcccf7f98da5df9d8a68e6b1b"
acc_3_secret = "keys/acc3_secret_key.pem"

map_hash_to_name = {
    acc_1_hash : acc_1_name,
    acc_2_hash : acc_2_name,
    acc_3_hash : acc_3_name
}

def run_command(cmd_with_args) -> str:
    result = subprocess.run(cmd_with_args , stdout=subprocess.PIPE)
    return result.stdout.decode()

def get_state_root_hash_from_response(res):
    js = json.loads(res)
    return js['result']['state_root_hash']

def get_state_root_hash():
    return (get_state_root_hash_from_response(run_command(['casper-client', 'get-state-root-hash' ,'--node-address' ,rpc_node])))

def get_contract_from_with_name(contract_name):
    cmd = ['casper-client','query-global-state' ,'--node-address' , rpc_node, '--state-root-hash',  get_state_root_hash(), '--key' ,f'account-hash-{acc_3_hash}', '-q', contract_name]
    return run_command(cmd)

def mint(metadata , amount , recipient , price,secret_key):
    metadata_js = json.loads(metadata)
    mint_cmd = f"casper-client put-deploy --node-address {rpc_node} --chain-name casper-test --secret-key {secret_key} --session-hash {contract_hash} --payment-amount 5300000000 --session-entry-point \"mint\" --session-arg \"metadata:String='{{\\\"name\\\" : \\\"{metadata_js['name']}\\\", \\\"token_uri\\\" : \\\"{metadata_js['token_uri']}\\\" , \\\"checksum\\\":\\\"{metadata_js['checksum']}\\\"}}'\" --session-arg \"price:u256='{price}'\" --session-arg \"amount:u64='{amount}'\" --session-arg \"recipient:key='account-hash-{recipient}'\""
    return mint_cmd

def publish_request(prod_hash, amount, holder_id, comission,secret_key):
    publish_req_cmd = f"casper-client put-deploy --node-address {rpc_node} --chain-name casper-test --secret-key {secret_key} --session-hash {contract_hash} --payment-amount 3500000000 --session-entry-point \"publish_request\" --session-arg \"producer-account:key='account-hash-{prod_hash}'\" --session-arg \"amount:u64='{amount}'\" --session-arg \"holder_id:u64='{holder_id}'\" --session-arg \"comission:u8='{comission}'\""
    return publish_req_cmd

def deploy_contract(acc_key):
    deploy_cmd = f"casper-client put-deploy -n {rpc_node} --chain-name casper-test --payment-amount 130420060000 -k {acc_key} --session-path deploy/contract.wasm"
    return deploy_cmd
def get_request_object(key):
    cmd = ['casper-client','get-dictionary-item' ,'--node-address' , rpc_node, '--state-root-hash',  get_state_root_hash(), "--dictionary-name" ,"request_objects", "--dictionary-item-key" , f"{key}" , "--contract-hash" , f"hash-{contract_hash}"]
    cmds = run_command(cmd)
    cmds_js = json.loads(cmds)
    obj = str(cmds_js['result']['stored_value']['CLValue']['parsed']).split(',')
    js_obj = {"holder_id" : obj[0] , "amount" : obj[1] , "comission" : obj[2], "producer" : obj[3] , "publisher" : obj[4]}
    return json.dumps(js_obj, indent=4)

def buy(amount , approved_id, secret_key, price):
    buy_cmd = f"casper-client put-deploy -n {rpc_node} --chain-name casper-test --payment-amount 15013050000 -k {secret_key} --session-path deploy/session.wasm --session-arg \"amount:u512='{int(amount)*int(price)}'\" --session-arg \"approved_id:u64='{approved_id}'\" --session-arg \"contract_hash:key='hash-{contract_hash}'\" --session-arg \"cnt:u64='{amount}'\""
    return buy_cmd

def approve_request(key,secret_key):
    cmd = f"casper-client put-deploy --node-address {rpc_node} --chain-name casper-test --secret-key {secret_key} --session-hash {contract_hash} --payment-amount 8931000000 --session-entry-point \"approve\" --session-arg \"request_id:u64='{key}'\""
    return cmd

def get_prod_reqs(account_hash):
    cmd = ['casper-client','get-dictionary-item' ,'--node-address' , rpc_node, '--state-root-hash',  get_state_root_hash(), "--dictionary-name" ,"producer_requests", "--dictionary-item-key" , f"{account_hash}" , "--contract-hash" , f"hash-{contract_hash}"]
    cmds = run_command(cmd)
    cmds_js = json.loads(cmds)
    lst = list(cmds_js['result']['stored_value']['CLValue']['parsed'])    
    return lst

def get_pub_reqs(account_hash):
    cmd = ['casper-client','get-dictionary-item' ,'--node-address' , rpc_node, '--state-root-hash',  get_state_root_hash(), "--dictionary-name" ,"publiser_requests", "--dictionary-item-key" , f"{account_hash}" , "--contract-hash" , f"hash-{contract_hash}"]
    cmds = run_command(cmd)
    cmds_js = json.loads(cmds)
    lst = list(cmds_js['result']['stored_value']['CLValue']['parsed'])    
    return lst

def get_token(token_id):
    cmd = ['casper-client','get-dictionary-item' ,'--node-address' , rpc_node, '--state-root-hash',  get_state_root_hash(), "--dictionary-name" ,"metadatas", "--dictionary-item-key" , f"{token_id}" , "--contract-hash" , f"hash-{contract_hash}"]
    cmds = run_command(cmd)
    cmds_js = json.loads(cmds)
    obj = str(cmds_js['result']['stored_value']['CLValue']['parsed']).split(',')
    #good3,sjducjfnbgjchfnfjdurjleoricjdnrj,ikojcjfnbgjchfnfjdurjdkieicjidue,10000000000
    #name,uri,checksum,price
    js_obj = {"name" : obj[0] , "token_uri" : obj[1] , "checksum" : obj[2], "price" : obj[3],"comission" : obj[4]}
    return json.dumps(js_obj, indent=4)

def get_holder_from_hex(hex):
    k = bytes.fromhex(hex)
    remaining_amount = int.from_bytes(k[0:8], byteorder='little')
    amount = int.from_bytes(k[8:16], byteorder='little')
    token_id = int.from_bytes(k[16:24], byteorder='little')
    return {"remaining_amount" : remaining_amount , "amount" : amount , "token_id" : token_id}

def get_holder_by_id(id):
    cmd = ['casper-client','get-dictionary-item' ,'--node-address' , rpc_node, '--state-root-hash',  get_state_root_hash(), "--dictionary-name" ,"holders", "--dictionary-item-key" , f"{id}" , "--contract-hash" , f"hash-{contract_hash}"]
    cmds = run_command(cmd)
    cmds_js = json.loads(cmds)
    obj = str(cmds_js['result']['stored_value']['CLValue']['bytes'])
    holder = get_holder_from_hex(obj)
    #get token_id and get token
    token = get_token(holder['token_id'])
    holder['token'] = json.loads(token)
    return json.dumps(holder , indent=4)

def get_holder_ids(account_hash):
    cmd = ['casper-client','get-dictionary-item' ,'--node-address' , rpc_node, '--state-root-hash',  get_state_root_hash(), "--dictionary-name" ,"owners", "--dictionary-item-key" , f"{account_hash}" , "--contract-hash" , f"hash-{contract_hash}"]
    cmds = run_command(cmd)
    cmds_js = json.loads(cmds)
    lst = list(cmds_js['result']['stored_value']['CLValue']['parsed'])    
    return lst

menu = colored("""
\------------ NFT Client Menu ------------/

    1) Deploy Contract (124.39383 CSPR)
    2) Mint token (~5 CSPR) 
    3) Publish Request (3.47974 CSPR)
    4) View Request Object
    5) View My incoming requests
    6) View My outgoing requests
    7) View My tokens
    8) Approve Request (5.83010 CSPR)
    9) Buy Token (15.01205 CSPR)
    10) Get Metadata by token_id
    11) Get Holder by holder_id
    0) exit

/------------ NFT Client Menu ------------\\
""" , "green", attrs=['bold'])
information = ""

def main():
    print(colored("Login using user [1-3] : ", "cyan"))
    user_number = input()
    account_secret = None
    account_hash = None
    account_name = None
    if user_number == "1":
        account_secret = acc_1_secret
        account_hash = acc_1_hash
        account_name = acc_1_name
    elif user_number == "2":
        account_secret = acc_2_secret
        account_hash = acc_2_hash
        account_name = acc_2_name
    elif user_number == "3":
        account_secret = acc_3_secret
        account_hash = acc_3_hash
        account_name = acc_3_name
    else:
        print(colored("Invalid choice", "red"))
        return

    while True:
        print(menu)
        choice = input(colored("Enter your choice :", "yellow", attrs=['bold', 'underline'])+" ")
        #if True:
        try:
            if choice == "1":
                print(colored("Copy the cmd :" , "yellow" , attrs=['underline']))
                print("\t"+colored(deploy_contract(account_secret),"magenta"))
            elif choice == "0":
                return
            elif choice == "-1":
                print(information)
            elif choice == "2":
                #get metadata file
                metadata_addr = input(colored("Metadata path : " , "green"))
                with open(metadata_addr,"r") as f:
                    metadata = f.read()
                amount = input(colored("Amount : " , "green"))
                recipient_ind = input(colored("Recipient [1-3] : " , "green"))
                recipient_hash = None
                if recipient_ind == "1":
                    recipient_hash = acc_1_hash
                elif recipient_ind == "2":
                    recipient_hash = acc_2_hash
                elif recipient_ind == "3":
                    recipient_hash = acc_3_hash
                price = input(colored("Price : " , "green"))
                print(colored("Copy the cmd :" , "yellow" , attrs=['underline']))
                print("\t",colored(mint(metadata , amount , recipient_hash , price,account_secret),"magenta"))
            elif choice == "3":
                comission = input(colored("Enter your expected comission [0-100]: ","green"))
                if int(comission)>100 or int(comission)<0:
                    print(colored("Invalid comission","red"))
                    continue
                holder_id = input(colored("Enter holder_id : ","green"))
                amount = input(colored("Enter amount to request: ","green"))
                prod_account_hash = input(colored("Enter producer's account-hash (without the 'account-hash-part'): ","green"))
                print(colored("Copy the cmd :" , "yellow" , attrs=['underline']))
                print("\t",colored(publish_request(prod_account_hash,amount , holder_id , comission , account_secret),"magenta"))
            elif choice == "4":
                key = input(colored("Enter the request_id : ","green"))
                print(colored(get_request_object(key),"cyan" , attrs=['bold']))
            elif choice == "5":
                lst = get_prod_reqs(account_hash)
                print(colored("Your incoming requests : ","green"))
                for i in lst:
                    print(colored(get_request_object(i),"cyan" , attrs=['bold']))
            elif choice == "6":
                lst = get_pub_reqs(account_hash)
                print(colored("Your outgoing requests : ","green"))
                for i in lst:
                    print(colored(get_request_object(i),"cyan" , attrs=['bold']))
            elif choice == "7":
                print(colored("Your tokens : ","green"))
                tokens = get_holder_ids(account_hash)
                i = 0
                for token in tokens:
                    print(colored(get_holder_by_id(token),"cyan" , attrs=['bold']))
                    i+=1
            elif choice == "8":
                lst = get_prod_reqs(account_hash)
                print(colored("Your incoming requests : ","green"))
                for i in lst:
                    print(colored(f"{i} : "+str(get_request_object(i)), "cyan" , attrs=['bold']))
                req_id = input(colored("Enter the request_id : ","green"))
                print(colored("Copy the cmd :","yellow" , attrs=['underline']))
                print(colored(approve_request(req_id,account_secret),"magenta"))
            elif choice == "9":
                amount = input(colored("Enter the amount : ","green"))
                approved_id = input(colored("Enter the approved_id : ","green"))
                price = input(colored("Enter the price : ","green"))
                print(colored("Copy the cmd :","yellow" , attrs=['underline']))
                print(colored(buy(amount,approved_id,account_secret,price),"magenta"))
            elif choice == "10":
                token_id = input(colored("Enter the token_id : ","green"))
                print(colored(get_token(token_id),"cyan" , attrs=['bold']))
            elif choice == "11":
                holder_id = input(colored("Enter the holder_id : ","green"))
                print(colored(get_holder_by_id(holder_id),"cyan" , attrs=['bold']))
        except:
            pass
main()