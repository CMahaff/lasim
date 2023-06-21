# Lemmy Account Settings Instance Migrator

TODO: Change "instance" to Url type and then use ".join" to add the API call
- See last example here: https://docs.rs/url/latest/url/

TODO: APIs to use to set data:
- <instance>/api/v3/community (GetCommunity request - to translate names to IDs)
- <instance>/api/v3/community/block (BlockCommunity request)
- <instance>/api/v3/community/follow (FollowCommunity request)
- <instance>/api/v3/user (GetPersonDetails request)
- <instance>/api/v3/user/block (BlockPerson request)
- <instance>/api/v3/user/save\_user\_settings (SaveUserSettings request)
