-module(maxBlockingHibernate).
-compile(export_all).

benchmark([N]) ->
    Start = erlang:system_time(millisecond),
    NumOfProcess = element(1, string:to_integer(N)),
    maxP(NumOfProcess),
    End = erlang:system_time(millisecond),
    io:format("Total time taken: ~f seconds~n", [(End-Start)/1000]).

maxP(N) when N>0 ->
    % io:format("Spawned~n"),
    spawn(?MODULE, hibernate, []),
    % register(list_to_atom(integer_to_list(N)),Pid),
    % io:format("~p~n", Pid),
    maxP(N-1);

maxP(0) ->
    io:format("Finished spawning processes.~n").

hibernate() ->
    erlang:hibernate(?MODULE, p, []).

p() ->
    receive
        "hello" ->
            io:format("Hello")
    end.

