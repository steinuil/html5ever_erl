-module(html5ever_nif).

-export([parse_html5/1]).

-include("cargo.hrl").
-on_load(init/0).
-define(NOT_LOADED, not_loaded(?LINE)).

parse_html5(_Doc) ->
    ?NOT_LOADED.

init() ->
    ?load_nif_from_crate(html5ever_nif, 0).

not_loaded(Line) ->
    erlang:nif_error({not_loaded, [{module, ?MODULE}, {line, Line}]}).
