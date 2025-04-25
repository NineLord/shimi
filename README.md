# Shimi

A Script of common things a developer might need.  
It contains commands that are too inconvenient to type every time, or just hard to remember.

## Road Map
1. Finish adding TTL aliases.
2. Implement `s d ip`/`s d port-forward`.
3. Implement `s d up`/`s d down`/`s d reset`, require loading `docker-compose`/`kubernetes` config files.
4. Allow overwriting default behaviors and saving them to the config file.
5. Allow adding custom commands.
6. Generate auto-completion file during compilation.
7. Continue to other commands . . .

## TODOs
* [ ] Changing the config wizard to manually editing specific settings  
instead of going over all of them each time running the config command.
* [ ] Allow to user to overwrite default behaviors using the config, such as:
 * Default command at `s docker exec`.
 * By default `s docker ps` not showing all containers.
 * By default `s docker logs` always follows.
 * By default `s docker logs` shows all the logs (no `tail` turned on).
* [ ] `clap` has feature for auto completions to commands.

#### License

<sup>
Licensed under either of <a href="LICENSE-APACHE">Apache License, Version
2.0</a> or <a href="LICENSE-MIT">MIT license</a> at your option.
</sup>

<br>

<sub>
Unless you explicitly state otherwise, any contribution intentionally submitted for inclusion in Shimi by you,<br>
as defined in the Apache-2.0 license, shall be dual licensed as above,<br>
without any additional terms or conditions.
</sub>