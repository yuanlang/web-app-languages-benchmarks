-module(testLimitHibernate).
-compile(export_all).

benchmark() ->
    {ok, F} = file:open("test_erl_hibernate_limit.txt", [write]),
    loop(F, 1).

loop(F, N) ->
    {_, Second, _} = os:timestamp(),
    io:format(F, "ts(sec): ~w proc no: ~w~n", [Second, N]),
    spawn(?MODULE, hibernate, []),
    % Info = erlang:process_info(Pid, current_function),
    % io:format("~w", [Info]).
    loop(F, N+1).

hibernate() ->
    erlang:hibernate(?MODULE, p, []).

p() ->
    receive
        "hello" ->
            io:format("Hello")
    end.
