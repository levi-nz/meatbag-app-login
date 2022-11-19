# meatbag-app-login
Login automation proof-of-concept for [meatbag.app](https://meatbag.app).

## Usage
The program, by default, reads combinations from `credentials.txt` in the `username:password` format.
There is only 1 thread by default, and once all combinations are tested, the program exits.
Valid and invalid combinations are logged (be sure to run with `RUST_LOG=info`).

This application does not have support for proxies by default. This is because this is simply a proof
of concept and I did not want to make it any easier than it already is for attackers. Proxy support can
be easily added, however.

Output using the default `credentials.txt` file and configuration:
```
[2022-11-19T15:18:00Z INFO  meatbag_app_login] Parsed 3 accounts from credentials.txt
[2022-11-19T15:18:00Z INFO  meatbag_app_login] Starting 1 login threads
[2022-11-19T15:18:01Z INFO  invalid_logins] test@gmail.com:hello is invalid
[2022-11-19T15:18:02Z INFO  invalid_logins] test1@gmail.com:hello1 is invalid
[2022-11-19T15:18:03Z INFO  invalid_logins] test2@gmail.com:hello2 is invalid
[2022-11-19T15:18:03Z INFO  meatbag_app_login] Done
```

## Motivation
This repository has came to be after I recently discovered Meatbag, a website claiming
to be "Twitter 2.0".

The website has a basic text captcha for registration, but offers no protection for login.
There does, however, appear to be an IP rate limit. This means that an attacker can login
potentially thousands of times (or even more) per second, while consuming very little resources,
to test leaked username + password combinations to perform an ATO (Account Takeover Attack).

Account Takeover Attacks are conducted by cybercriminals to test usernames and passwords to
see if they are valid. The usernames and passwords are usually obtained from leaked databases.

Most websites have protection against ATO's with the use of an anti-bot service, like reCAPTCHA.

## Technical
The website makes an HTTP request to `https://meatbag.app/oauth/token` with the username and password
when a user attempts to login. A request made from a legitimate user looks like this:

```http request
POST https://meatbag.app/oauth/token HTTP/2.0

content-length: 274
sec-ch-ua: "Google Chrome";v="107", "Chromium";v="107", "Not=A?Brand";v="24"
accept: application/json, text/plain, */*
content-type: application/json
sec-ch-ua-mobile: ?0
user-agent: Mozilla/5.0 (Windows NT 10.0; Win64; x64) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/107.0.0.0 Safari/537.36
sec-ch-ua-platform: "Windows"
origin: https://meatbag.app
sec-fetch-site: same-origin
sec-fetch-mode: cors
sec-fetch-dest: empty
referer: https://meatbag.app/
accept-encoding: gzip, deflate, br
accept-language: en-US,en;q=0.9

{"client_id":"Pt-G_VrDcoVhc32ZpuqMf4FHmDW37bxPtcjQm7J2k6w","client_secret":"8PvfgVS5UFStXXq-LR5PfcAawU_524ZQraTjOuXw0n8","redirect_uri":"urn:ietf:wg:oauth:2.0:oob","grant_type":"password","username":"test@gmail.com","password":"hello","scope":"read write follow push admin"}
```

At first glance, most information here is junk. The important part is the request body:

```json
{
  "client_id": "Pt-G_VrDcoVhc32ZpuqMf4FHmDW37bxPtcjQm7J2k6w",
  "client_secret": "8PvfgVS5UFStXXq-LR5PfcAawU_524ZQraTjOuXw0n8",
  "redirect_uri": "urn:ietf:wg:oauth:2.0:oob",
  "grant_type": "password",
  "username": "test@gmail.com",
  "password": "hello",
  "scope":"read write follow push admin"
}
```
When logging in with the username `test@gmail.com` and the password `hello`, we can clearly see the credentials
in plaintext sent in the request. Since the username and password is all we need for the request, we can
actually make a request like this that will still work:

```http request
POST https://meatbag.app/oauth/token HTTP/2.0

content-type: application/json
accept: */*

{
	"client_id": "",
	"client_secret": "",
	"redirect_uri": "urn:ietf:wg:oauth:2.0:oob",
	"grant_type": "password",
	"username": "test@gmail.com",
	"password": "hello",
	"scope": "read write follow push admin"
}
```

With this information in hand, we can make a program that automates the login process.
This is what this repository does.

Because there is no protection on the login form, this allows any cybercriminal to easily
automate the process with essentially an unlimited number of credentials to find combinations that work,
record the working combinations, and perform an Account Takeover Attack. This is a huge deal as it also
allows an attacker to take as many guesses as they wish to guess someone's password. This also applies to 2FA codes;
although it is hard to guess a six-digit code, it becomes easier when you have infinite guesses.

Although the login endpoint appears to have IP-based rate limiting, this is not a solution. Attackers
often have access to proxies, essentially allowing them to change their IP address. These proxies often look
like a legitimate residential IP address, which evades many proxy detection systems.

## Solution
The easiest solution to prevent login automation would be to implement reCAPTCHA. reCAPTCHA has an invisible
challenge (v3 and Enterprise), meaning a legitimate user does not need to complete a captcha to login.
Most modern anti-bot services are invisible in this way, which involves a background challenge that is hard
for an attacker to re-create to make it look like the HTTP request originates from a legitimate user.

reCAPTCHA Enterprise is the most common and affordable solution that significantly increases the difficulty
and compute costs for attackers, however there are most sophisticated (and significantly more expensive) solutions
like [F5 Distributed Cloud Bot Defense](https://www.f5.com/cloud/products/bot-defense).

Bots are always evolving and there is no definite solution to the problem, but the difficulty can be
significantly increased, and the compute costs for an attacker can also be significantly increased,
which essentially means attackers can only test a certain number of logins per second.

### Disclaimer
I do not condone the use of ATO's (Account Takeover Attacks) or credential stuffing.
This repository is simply a proof of concept to prove that it is possible to automate
the process of logging in programmatically, which can be used by attackers to test
potentially thousands of username + password combinations to perform account takeovers.