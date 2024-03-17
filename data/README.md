# Data used by simple-protocols

## Quotes

`quotes.txt` contains some plain-text ASCII-only quotes, one per line.
This data is used by the quote of the day protocol.
Quotes can be a maximum of 510 bytes and are limited to the characters `!#"$%&'()*+,-./ 0123456789:;<=>?ABCDEFGHIJKLMNOPQRSTUVWXYZ_abcdefghijklmnopqrstuvwxyz~`.
To speed up builds of simple-protocols, the data is no longer fetched from the API at build-time.
The current contents of the file are from <https://api.quotable.io/>, retrieved on 2024-03-15.

## User Info

`users.json` contains sample user information, including username, full name, and extra info.
This data is used by the finger protocol and the active users protocol.
Usernames are limited to the characters `-0123456789ABCDEFGHIJKLMNOPQRSTUVWXYZ_abcdefghijklmnopqrstuvwxyz`.
The user's full name is limited to UTF-8 (but if possible should be ASCII-only), and can not contain CR (`\r`) or LF (`\n`).
The extra user information is limited to UTF-8 (but if possible should be ASCII-only), and may contain multiple lines.
Line endings are automatically adjusted by the build script.
The usernames and full names in the file are based on <https://en.wikipedia.org/wiki/Alice_and_Bob#Cast_of_characters>.
