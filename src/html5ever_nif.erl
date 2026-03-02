-module(html5ever_nif).
-moduledoc """
Basic Erlang bindings to html5ever.
""".

-export([parse_html5/1]).

-export_type([
    id/0,
    option/1,
    parent/0,
    qualified_name/0,
    doctype/0,
    processing_instructions/0,
    text/0,
    comment/0,
    attributes/0,
    element/0,
    xml_node/0,
    document/0
]).

-include("cargo.hrl").
-on_load(init/0).

-type id() :: {id, integer()}.

-type option(T) :: {some, T} | none.

-type parent() :: option(id()).

-type qualified_name() ::
    {qualified_name, Prefix :: option(binary()), Namespace :: binary(), Local :: binary()}.

-type doctype() ::
    {doctype, Id :: id(), Parent :: parent(), Name :: binary(), PublicId :: binary(),
        SystemId :: binary()}.

-type processing_instructions() ::
    {processing_instructions, Id :: id(), Parent :: parent(), Target :: binary(),
        Contents :: binary()}.

-type text() :: {text, Id :: id(), Parent :: parent(), Contents :: binary()}.

-type comment() :: {comment, Id :: id(), Parent :: parent(), Contents :: binary()}.

-type attributes() :: #{binary() := binary()}.

-type element() ::
    {element, Id :: id(), Parent :: parent(), Name :: qualified_name(), Attributes :: attributes(),
        Children :: [id()]}.

-type xml_node() :: doctype() | processing_instructions() | text() | comment() | element().

-doc """
A flat representation of an HTML document.
""".
-type document() ::
    {document, NodeCount :: integer(), Roots :: [id()], Nodes :: #{id() := xml_node()}}.

-doc """
Parse a UTF-8 binary to a `document()`.

## Raises

- `badarg` if `Doc` is not a valid UTF-8 binary.
""".
-spec parse_html5(binary()) -> document().
parse_html5(_Doc) ->
    not_loaded(?LINE).

init() ->
    ?load_nif_from_crate(html5ever_nif, 0).

not_loaded(Line) ->
    erlang:nif_error({not_loaded, [{module, ?MODULE}, {line, Line}]}).
