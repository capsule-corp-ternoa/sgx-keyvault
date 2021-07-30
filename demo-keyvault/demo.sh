!/bin/bash

# name of the docker image
DOCKER_IMAGE=ternoa/demo

# clone the rust-sgx-sdk (used to run the substratee-worker in the docker)
git clone https://github.com/baidu/rust-sgx-sdk.git

# get the substraTEE docker image from docker hub
docker pull $DOCKER_IMAGE

# if you want to build the docker image yourself, use the following command:
# docker build -t substratee -f DockerfileM5 .

# prepare the docker specific network
docker network rm ternoa-net
docker network create --subnet 192.168.10.0/24 ternoa-net

# prepare the output log directory
mkdir -p output

# start the tmux session
SESSION=sgxKeyvaultDemo
tmux has-session -t $SESSION

if [ $? != 0 ]
then
    tmux -2 new -d -s $SESSION -n "ternoa sgx keyvault demo"

    # create a window split by 4
    tmux split-window -v
    tmux split-window -h
    tmux select-pane -t 0
    tmux split-window -h

    # enable pane titles
    tmux set -g pane-border-status top

    # set length of left status to 50
    tmux set -g status-left-length 50

    # color the panes
    tmux select-pane -t 1 -P 'fg=colour073' # node
    tmux select-pane -t 2 -P 'fg=colour011' # client
    tmux select-pane -t 3 -P 'fg=colour043' # worker 1
    tmux select-pane -t 4 -P 'fg=colour043' # worker 1

    # start the substratee-node in pane 1
    tmux send-keys -t1 "docker run -ti \
        --ip=192.168.10.10 \
        --network=ternoa-net \
        -v $(pwd)/output:/ternoa/output \
        $DOCKER_IMAGE \
        \"/ternoa/start_node.sh\"" Enter

#     # run the ternoa demo in pane 2
#     tmux send-keys -t2 "docker run -ti \
#         --ip=192.168.10.30 \
#         --network=ternoa-net \
#         -v $(pwd)/output:/ternoa/output \
#         $DOCKER_IMAGE \
#         \"/ternoa/demo_scenarios.sh\"" Enter

    # start the ternoa-worker in pane 3
    tmux send-keys -t3 "docker run -ti \
        --ip=192.168.10.21 \
        --network=ternoa-net \
        -v $(pwd)/output:/ternoa/output \
        $DOCKER_IMAGE \
        \"/ternoa/start_worker.sh 2910\"" Enter

#     # start the 2d ternoa-worker in pane 4
#     tmux send-keys -t4 "docker run -ti \
#         --ip=192.168.10.21 \
#         --network=ternoa-net \
#         -v $(pwd)/output:/ternoa/output \
#         $DOCKER_IMAGE \
#         \"/ternoa/start_worker.sh 2911\"" Enter

fi

# Attach to session
tmux attach -t $SESSION