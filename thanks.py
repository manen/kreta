import base64, os, hashlib, secrets, requests
from bs4 import BeautifulSoup

def generate_stateOrNonce():
    return secrets.token_urlsafe(16)

def generate_code_challenge():
    global verifier

    verifier = base64.urlsafe_b64encode(os.urandom(64)).rstrip(b"=").decode("utf-8")
    digest = hashlib.sha256(verifier.encode("ascii")).digest()
    return base64.urlsafe_b64encode(digest).rstrip(b"=").decode("utf-8")

headers={
    "User-Agent":"Mozilla/5.0 (Linux; Android 10; K) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/124.0.0.0 Mobile Safari/537.36",
    "Content-Type": "application/x-www-form-urlencoded",
}

def Login(UserName, Password, InstituteCode):
    session = requests.Session()
    session.headers.update(headers)

    r=session.get(f"https://idp.e-kreta.hu/connect/authorize?redirect_uri=https://mobil.e-kreta.hu/ellenorzo-student/prod/oauthredirect&client_id=kreta-ellenorzo-student-mobile-android&response_type=code&prompt=login&state={generate_stateOrNonce()}&nonce={generate_stateOrNonce()}&scope=openid email offline_access kreta-ellenorzo-webapi.public kreta-eugyintezes-webapi.public kreta-fileservice-webapi.public kreta-mobile-global-webapi.public kreta-dkt-webapi.public kreta-ier-webapi.public&code_challenge={generate_code_challenge()}&code_challenge_method=S256", headers=headers)

    r2 = session.get(r.url)
    soup = BeautifulSoup(r2.text, "html.parser")
    ReturnUrl = soup.find("input", {"id": "ReturnUrl"})["value"]
    __RequestVerificationToken = soup.find("input", {"name": "__RequestVerificationToken"})["value"]
    body = {
        "ReturnUrl": ReturnUrl,
        "__RequestVerificationToken": __RequestVerificationToken,
        "UserName": UserName,
        "Password": Password,
        "InstituteCode": InstituteCode,
        "loginType":"InstituteLogin",
        "ClientId":"",
        "IsTemporaryLogin":"False"
    }
    r3 = session.post("https://idp.e-kreta.hu/account/login", data=body)
    r4=session.get(f"https://idp.e-kreta.hu{ReturnUrl}")
    connectTokenBody= {
        "code":r4.url.split("code=")[1].split("&")[0],
        "grant_type":"authorization_code",
        "redirect_uri":"https://mobil.e-kreta.hu/ellenorzo-student/prod/oauthredirect",
        "code_verifier":verifier,
        "client_id":"kreta-ellenorzo-student-mobile-android",

    }
    r5=requests.post("https://idp.e-kreta.hu/connect/token", headers={"User-Agent":"hu.ekreta.student/5.8.0+2025082301/SM-S9280/9/28","Content-Type":"application/x-www-form-urlencoded"}, data=connectTokenBody)
    return r5.json()
