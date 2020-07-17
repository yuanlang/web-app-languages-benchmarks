-module(testLimit).
-compile(export_all).

benchmark() ->
    {ok, F} = file:open("test_erl_limit.txt", [write]),
    loop(F, 1).

loop(F, N) ->
    io:format(F, "Number of process(es): ~w~n", [N]),
    spawn(?MODULE, p, []),
    loop(F, N+1).

p() ->
    receive
        "hello" ->
            io:format("Hello")
    end.
