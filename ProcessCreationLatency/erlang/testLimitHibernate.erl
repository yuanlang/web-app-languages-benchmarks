-module(testLimitHibernate).
-compile(export_all).

benchmark() ->
    loop(1).

loop(N) ->
    io:format("Number of process(es): ~w ~n", [N]),
    spawn(?MODULE, hibernate, []),
    % Info = erlang:process_info(Pid, current_function),
    % io:format("~w", [Info]).
    loop(N+1).

hibernate() ->
    erlang:hibernate(?MODULE, p, []).

p() ->
    receive
        "hello" ->
            io:format("Hello")
    end.
