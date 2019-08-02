# ESI client for Rust

A rather crudely-conceived ESI client for the Rust programming language.

## Development

This project is designed to be able to be built and validated offline. Test
data is downloaded from ESI using `curl` and then validated. These validations
may run without the first step of using `curl`. To update the data you will
need a valid ESI application with all scopes enabled, available at
[developers.eveonline.com][devev]. Copy the configuration template at
`secrets.toml` to `.secrets.toml` and fill in the missing information. *Never
commit `.secrets.toml`!*

When adding new test data, please ensure that it has been obfuscated to a level
that makes you comfortable. EVE is a dark, unforgiving place, and your enemies
may use any information you give them!

## License

I want you to be able to use this software regardless of who you may be, what
you are working on, or the environment in which you are working on it - I
hope you'll use it for good and not evil! To this end, ESIRS is licensed under
the [2-clause BSD][2cbsd] license, with other licenses available by request.
Happy coding!

[devev]: https://developers.eveonline.com/applications
[2cbsd]: https://opensource.org/licenses/BSD-2-Clause
