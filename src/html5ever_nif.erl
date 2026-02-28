-module(html5ever_nif).

-export([add/2]).

-include("cargo.hrl").
-on_load(init/0).
-define(NOT_LOADED, not_loaded(?LINE)).

add(_A, _B) ->
    ?NOT_LOADED.

init() ->
    ?load_nif_from_crate(html5ever_nif, 0).

not_loaded(Line) ->
    erlang:nif_error({not_loaded, [{module, ?MODULE}, {line, Line}]}).
