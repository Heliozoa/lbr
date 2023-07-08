Thanks for your interest in contributing to LBR! Issues, pull requests and discussion threads are welcome.


## Project structure

The project is split into a few crates in the `./crates` directory:

- `lbr_api` (`./crates/api`): Contains types for communicating between the backend and frontend.
- `lbr_server` (`./crates/backend`): A web server offering the functionality of LBR in its endpoints.
- `lbr_core` (`./crates/core`): Contains some core types shared by the main library and frontends.
- `lbr_web` (`./crates/frontend`): A web frontend that works with the backend.
- `jadata` (`./crates/jadata`): Generates files used by LBR related to the Japanese language.
- `lbr` (`./crates/lbr`): The main library of the project that offers all the core functionality.


## Setting up LBR locally

### Prepare data files
1. Download the files listed in `./jadata/README.md` to `./jadata/external`
2. Run `bash ./scripts/jadata.bash`

### Database setup
1. In psql, run `CREATE ROLE lbr WITH CREATEDB PASSWORD 'lbr';`
2. Download a database dump from https://github.com/tshatrov/ichiran/releases and rename it to `./data/ichiran.pgdump`
3. Run `bash ./scripts/init-lbr-db.bash`
4. Run `bash ./scripts/init-ichiran-db.bash`
5. Run `bash ./scripts/generate-ichiran-schema.bash`


### Setting up ichiran
See https://readevalprint.tumblr.com/post/639359547843215360/ichiranhome-2021-the-ultimate-guide

0. Make sure the ichiran database has been set up in the previous step
1. Install [SBCL](http://sbcl.org/)
2. Download [quicklisp](https://www.quicklisp.org/beta/)
3. Execute `sbcl --load ./quicklisp.lisp`
4. Run `sbcl --eval "(quicklisp-quickstart:install)" --eval "(ql:add-to-init-file)" --eval "(sb-ext:quit)"`
5. Run `git clone https://github.com/tshatrov/ichiran ~/quicklisp/local-projects/ichiran`
6. Run `git clone https://gitlab.com/yamagoya/jmdictdb.git ~/jmdictdb`
7. Inside `~/quicklisp/local-projects/ichiran`, rename `settings.lisp.template` to `settings.lisp`
8. Inside `settings.lisp`, change `(defparameter *connection* '("jmdict" "postgres" "password" "localhost"))` to `(defparameter *connection* '("ichiran" "lbr" "lbr" "localhost"))`
9. Inside `settings.lisp`, change `(defparameter *jmdict-data* #p"/home/you/dump/jmdict-data/")` to `(defparameter *jmdict-data* #p"/home/YOUR_USERNAME_HERE/jmdictdb/data/")`
10. Run `sbcl --eval "(ql:quickload :ichiran)" --eval "(ichiran/mnt:add-errata)" --eval "(ichiran/test:run-all-tests)" --eval "(sb-ext:quit)"`. All the tests should pass.
11. Run `sbcl --eval "(ql:quickload :ichiran/cli)" --eval "(ichiran/cli:build)" --eval "(sb-ext:quit)"`, execute.
12. Give the CLI a go with `./ichiran-cli -f "こんばんは。"`.
13. Run `mv ~/quicklisp/local-projects/ichiran/ichiran-cli ./data/ichiran-cli`.
14. You can remove `~/quicklisp` and `~/jmdictdb` if you'd like.


## Development

LBR uses the nightly toolchain.

### Formatting
`cargo fmt`

### Linting
`cargo clippy`

### Running the project
Run `bash ./scripts/dev.bash`

The backend will start at `http://0.0.0.0:3000` and the frontend will be available at `http://0.0.0.0:8080`.
