# Check the manual failover

source "../tests/includes/init-tests.tcl"

xtest "Create a 5 nodes cluster" {
    create_cluster 5 5
}

xtest "Cluster is up" {
    assert_cluster_state ok
}

xtest "Cluster is writable" {
    cluster_write_test 0
}

xtest "Instance #5 is a slave" {
    assert {[RI 5 role] eq {slave}}
}

xtest "Instance #5 synced with the master" {
    wait_for_condition 1000 50 {
        [RI 5 master_link_status] eq {up}
    } else {
        fail "Instance #5 master link status is not up"
    }
}

set current_epoch [CI 1 cluster_current_epoch]

set numkeys 50000
set numops 10000
set cluster [redis_cluster 127.0.0.1:[get_instance_attrib redis 0 port]]
catch {unset content}
array set content {}

xtest "Send CLUSTER FAILOVER to #5, during load" {
    for {set j 0} {$j < $numops} {incr j} {
        # Write random data to random list.
        set listid [randomInt $numkeys]
        set key "key:$listid"
        set ele [randomValue]
        # We write both with Lua scripts and with plain commands.
        # This way we are able to stress Lua -> Redis command invocation
        # as well, that has tests to prevent Lua to write into wrong
        # hash slots.
        if {$listid % 2} {
            $cluster rpush $key $ele
        } else {
           $cluster eval {redis.call("rpush",KEYS[1],ARGV[1])} 1 $key $ele
        }
        lappend content($key) $ele

        if {($j % 1000) == 0} {
            puts -nonewline W; flush stdout
        }

        if {$j == $numops/2} {R 5 cluster failover}
    }
}

xtest "Wait for failover" {
    wait_for_condition 1000 50 {
        [CI 1 cluster_current_epoch] > $current_epoch
    } else {
        fail "No failover detected"
    }
}

xtest "Cluster should eventually be up again" {
    assert_cluster_state ok
}

xtest "Cluster is writable" {
    cluster_write_test 1
}

xtest "Instance #5 is now a master" {
    assert {[RI 5 role] eq {master}}
}

xtest "Verify $numkeys keys for consistency with logical content" {
    # Check that the Redis Cluster content matches our logical content.
    foreach {key value} [array get content] {
        assert {[$cluster lrange $key 0 -1] eq $value}
    }
}

xtest "Instance #0 gets converted into a slave" {
    wait_for_condition 1000 50 {
        [RI 0 role] eq {slave}
    } else {
        fail "Old master was not converted into slave"
    }
}

## Check that manual failover does not happen if we can't talk with the master.

source "../tests/includes/init-tests.tcl"

xtest "Create a 5 nodes cluster" {
    create_cluster 5 5
}

xtest "Cluster is up" {
    assert_cluster_state ok
}

xtest "Cluster is writable" {
    cluster_write_test 0
}

xtest "Instance #5 is a slave" {
    assert {[RI 5 role] eq {slave}}
}

xtest "Instance #5 synced with the master" {
    wait_for_condition 1000 50 {
        [RI 5 master_link_status] eq {up}
    } else {
        fail "Instance #5 master link status is not up"
    }
}

xtest "Make instance #0 unreachable without killing it" {
    R 0 deferred 1
    R 0 DEBUG SLEEP 10
}

xtest "Send CLUSTER FAILOVER to instance #5" {
    R 5 cluster failover
}

xtest "Instance #5 is still a slave after some time (no failover)" {
    after 5000
    assert {[RI 5 role] eq {master}}
}

xtest "Wait for instance #0 to return back alive" {
    R 0 deferred 0
    assert {[R 0 read] eq {OK}}
}

## Check with "force" failover happens anyway.

source "../tests/includes/init-tests.tcl"

xtest "Create a 5 nodes cluster" {
    create_cluster 5 5
}

xtest "Cluster is up" {
    assert_cluster_state ok
}

xtest "Cluster is writable" {
    cluster_write_test 0
}

xtest "Instance #5 is a slave" {
    assert {[RI 5 role] eq {slave}}
}

xtest "Instance #5 synced with the master" {
    wait_for_condition 1000 50 {
        [RI 5 master_link_status] eq {up}
    } else {
        fail "Instance #5 master link status is not up"
    }
}

xtest "Make instance #0 unreachable without killing it" {
    R 0 deferred 1
    R 0 DEBUG SLEEP 10
}

xtest "Send CLUSTER FAILOVER to instance #5" {
    R 5 cluster failover force
}

xtest "Instance #5 is a master after some time" {
    wait_for_condition 1000 50 {
        [RI 5 role] eq {master}
    } else {
        fail "Instance #5 is not a master after some time regardless of FORCE"
    }
}

xtest "Wait for instance #0 to return back alive" {
    R 0 deferred 0
    assert {[R 0 read] eq {OK}}
}
