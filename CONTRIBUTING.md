# Contribution guidelines

### Code style

please use sonarlint and respect it as much as possible.
always add comment on why you did this or that (I know I'm the first not doing it, but it helps everyone)

### Naming convention

please respect https://github.com/rust-lang/rfcs/blob/master/text/0430-finalizing-naming-conventions.md. \
if you create structure from the api not respecting this naming convention is not an isssue since it seem that changing
the name from bannerImage to banner_image make it stop working. if this is something that i missed and it's really
possible
please tell me and i change it.

### When adding new functionality to the bot

first focus on a working version and after do it more properly and clean (except if you want to do it from the start)
since all new functionality will not be accepted or added having a "prototype" is best.

### When editing an already existing functionality

please tell what have changed and the impact better performance, more readable, using new api.