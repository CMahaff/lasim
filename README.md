# Lemmy Account Settings Instance Migrator

![LASIM Screenshot](LASIM.PNG)

## How it works

1. Create an account on the server you want to migrate to.
2. Run LASIM, enter your old account credentials, and hit "Download". Your information is saved to a local file.
3. In LASIM, hit the Upload tab, then enter your new account credentials and hit "Upload".
4. The local file is used to update your new account's blocked users, blocked communities, followed communities, and *most* profile settings.
5. That's it!

## Additional Information
- The following profile settings are not modified by LASIM: your avatar image, your banner image, your display name, your email, your bio, your Matrix user, and your 2-Factor token
    - All other profile settings will match your old account
- LASIM is otherwise additive - it will add any  blocked users, blocked communities, and/or followed communities present in your old account, but not in your new account, but will NOT delete any blocked users, blocked communities, and/or followed communities already present in your new account
- LASIM will automatically detect if your new account already has some of the blocked users, blocked communities, and/or followed communities and will not re-issue those API calls
- LASIM respects the API rate limits set by your instance owner, so some servers may take longer than others
- LASIM will skip entries that fail to apply - re-run LASIM to try these entries again
- LASIM has to make numerous API calls to migrate everything - be patient
- This should go without saying, but obviously both your new and old accounts are still distinct - LASIM simply makes it easier to move from one to the other

## Limitations
- Versions of LASIM only target specific Lemmy BE API versions, which are currently changing rapidly. See the Version Support table.
    - As long as the "Profile Version" is the same between LASIM versions, it is possible to use different LASIM versions together to target Lemmy servers running different incompatible API versions.
    - At time of writing there is planned support, but no code written, to support migration from older Profile Versions to newer ones.
    - If your version is not explicitely listed, take the latest LASIM that is available!

## Version Support
| LASIM Version | LASIM Profile Version | Supported Lemmy BE API Version(s) |
| ------------- | --------------------- | --------------------------------- |
| 0.1.\*        | 1                     | 0.18.1 (rc.9+), 0.18.2            |
| 0.2.\*        | 2                     | 0.18.3 (rc.5+) and above          |
