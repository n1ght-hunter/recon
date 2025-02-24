--!strict

local recon_plugin = require "library/recon-plugin"

function ispositive(x: number): boolean
    return x > 0
end

print(ispositive(1))


function isfoo(a)
    return recon_plugin.sum(a, 1)
end

print(isfoo(1))
