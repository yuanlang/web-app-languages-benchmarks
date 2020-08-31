-module(testLimit).
-compile(export_all).

benchmark() ->
    {ok, F} = file:open("test_erl_limit.txt", [write]),
    loop(F, 1).

loop(F, N) ->
    {_, Second, _} = os:timestamp(),
    io:format(F, "ts(sec): ~w proc no: ~w~n", [Second, N]),
    spawn(?MODULE, p, []),
    loop(F, N+1).

p() ->
    receive
        "hello" ->
            io:format("Hello")
    end.
