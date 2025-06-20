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
* __New features__:
  * [ ] Change `Kubernetes` to `OpenShift` and add `Kubernetes`.
  * [ ] Add `Feeling lucky` mode, which guess the container you asked for without user prompts.
    * The algorithm will save in his own config, the last known `docker ps` state, as `HashMap` between service (from title from `docker-compose.yml`) to his actual container(__s__).
	* According to this config, it will be able to "add fake" aliases to containers with their "count" number at the end.
* __Config related__:
  * [ ] Allow to user to overwrite default behaviors using the config (can be done with [clap-config-file](https://crates.io/crates/clap-config-file)), such as:
    * Default command at `s docker exec`.
    * By default `s docker ps` not showing all containers.
    * By default `s docker logs` always follows.
    * By default `s docker logs` shows all the logs (no `tail` turned on).
  * [ ] Add to the config, each container can have his own "shell" to be used when doing `docker exec`, to be able to overwrite `/bin/bash` default.
* __UX related__:
  * [ ] `clap` has feature for auto completions to commands.
    * A different example how to achieve shell completion with [clap_generate](https://github.com/tgm-templates/rust-cli/tree/master).
  * [ ] `dialoguer` has feature for history to prompts.
  * [ ] Add better support for unexpected panics with [human-panic](https://github.com/rust-cli/human-panic).
  * [ ] Add demo to this readme using [Terminalizer](https://github.com/faressoft/terminalizer).
  * [ ] Add support to multi-lang using [rust-i18n](https://crates.io/crates/rust-i18n/)/[sys-locale](https://crates.io/crates/sys-locale).
  * [ ] Print more human readable numbers/dates/etc with [readable](https://crates.io/crates/readable).
* __Testing related__:
  * [ ] Try mocking library to make testing easier with [injectorpp](https://www.reddit.com/r/rust/comments/1l2qhb6/a_new_mocking_library_to_mock_functions_without/).
* __Bug fixes__:
  * [ ] Dates (TTLs) currently read while ignoring the time zone, maybe can be fixed with [dateparser](https://crates.io/crates/dateparser)/[parse_datetime](https://crates.io/crates/parse_datetime).
  * [ ] If the user doesn't have permissions to run docker, he gets a bad error message, should replace it with: "Run: `sudo usermod -aG docker $USER` and restart you terminal to add your use to user group that able to run docker commands, to make sure it works run `groups` and see that there is `docker` in the output".

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