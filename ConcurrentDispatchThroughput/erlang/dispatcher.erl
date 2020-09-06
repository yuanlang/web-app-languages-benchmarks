%%
%% %CopyrightBegin%
%%
%% Copyright Lang Yuan 2020. All Rights Reserved.
%%
%% The contents of this file are subject to the Erlang Public License,
%% Version 1.1, (the "License"); you may not use this file except in
%% compliance with the License. You should have received a copy of the
%% Erlang Public License along with this software. If not, it can be
%% retrieved online at http://www.erlang.org/.
%%
%% Software distributed under the License is distributed on an "AS IS"
%% basis, WITHOUT WARRANTY OF ANY KIND, either express or implied. See
%% the License for the specific language governing rights and
%% limitations under the License.
%%
%% %CopyrightEnd%

%% Benchmark Description:
%% A benchmark about message proxying through a dispatcher. The benchmark
%% spawns a certain number of receivers, one dispatcher, and a certain 
%% number of generators. The dispatcher forwards the messages that it 
%% receives from generators to the appropriate receiver. Each generator 
%% sends a number of messages to a specific receiver. The parameters of 
%% the benchmark are the number of receivers, the number of messages and 
%% the message MsgLen.
%%

%% Author  : Lang Yuan
%% Created : 1 Sep 2020 by Lang Yuan

-module(dispatcher).

% -export([bench_args/2, benchmark/1]).

% bench_args(Version, Conf) ->
%     {_,Cores} = lists:keyfind(number_of_cores, 1, Conf),
%     [F1, F2, F3] = case Version of
%         short -> [2, 16, 32];  
%         intermediate -> [2, 32, 40];  
%         long -> [5, 16, 32]
%     end,
%     [[RecvNum,GenNum,MsgLen] || RecvNum <- [F1 * Cores], GenNum <- [F2 * Cores], MsgLen <- [F3 * Cores]].

-export([benchmark/1]).

benchmark(Args) ->
	[RecvNum1,GenNum1,MsgLen1|_] = Args,
	RecvNum = list_to_integer(RecvNum1),
	GenNum  = list_to_integer(GenNum1),
	MsgLen  = list_to_integer(MsgLen1),

    Recvs = setup_receivers(RecvNum),
    Disp  = setup_dispatcher(),
    Gens  = setup_generators(Recvs, Disp, GenNum, MsgLen),
    Start = erlang:system_time(millisecond),

    [Pid ! {self(), do} || Pid <- Gens],
    %wait for the processes to finish
    % timer:sleep(1000*6),
    [receive {Pid, done} -> ok end || Pid <- Recvs],
    Disp ! {self(), done},
    receive
        {_, done} ->
            End = erlang:system_time(millisecond),
			io:format("Time taken to do the work: ~f seconds~n", [(End-Start)/1000]),
			ok
    end,
    ok.

%% setups

setup_receivers(RecvNum) -> setup_receivers(RecvNum, self(), []).

setup_receivers(0, _, Out) -> Out;
setup_receivers(RecvNum, Pid, Out) -> 
    setup_receivers(RecvNum - 1, Pid, [spawn_link(fun() -> receiver(Pid) end)|Out]).

setup_dispatcher() ->
    Me = self(),
    spawn_link(fun() -> dispatcher(Me, 0) end).

setup_generators(Recvs, Disp, GenNum, MsgLen) ->
    setup_generators(Recvs, Disp, self(), GenNum, MsgLen, []).

setup_generators([],_,  _, _, _, Out) -> Out;
setup_generators([Recv|Recvs], Disp, Pid, GenNum, MsgLen, Out) ->
    setup_generators(Recvs, Disp, Pid, GenNum, MsgLen, [spawn_link(fun() -> generator(Recv, Disp, Pid, GenNum, MsgLen) end) | Out]).

%% processes

receiver(Master) ->
    receive
        {_, done} -> Master ! {self(), done};
        {_, _} -> receiver(Master)
    end.

dispatcher(Master, N) ->
    receive
		{Master, done}  -> io:format("Dispatched msg count: ~w~n", [N]), Master ! {self(), done}, ok;
        {Pid, To, Data} -> To ! {Pid, Data},
                            dispatcher(Master, N+1)
    end.

generator(Recv, Disp, Master, GenNum, MsgLen) ->
    Data = lists:seq(1, MsgLen),
    receive
		{Master, done} -> ok;
        {Master, do} -> generator_push_loop(Recv, Disp, GenNum, Data);
        {Master, do, NewN} -> generator_push_loop(Recv, Disp, NewN, Data)
    end.

generator_push_loop(Recv, Disp, 0, _) ->
    Disp ! {self(), Recv, done};
generator_push_loop(Recv, Disp, GenNum, Data) ->
    Disp ! {self(), Recv, Data},
    generator_push_loop(Recv, Disp, GenNum-1, Data).

%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%5

