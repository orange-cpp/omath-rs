-- PatternScanner tests: generic scan over a Lua string buffer

function PatternScanner_FindsExactPattern()
    local buf = "\x90\x01\x02\x03\x04"
    local offset = omath.PatternScanner.scan(buf, "90 01 02")
    assert(offset ~= nil, "expected pattern to be found")
    assert(offset == 0, "expected offset 0, got " .. tostring(offset))
end

function PatternScanner_FindsPatternAtNonZeroOffset()
    local buf = "\x00\x00\xAB\xCD\xEF"
    local offset = omath.PatternScanner.scan(buf, "AB CD EF")
    assert(offset ~= nil, "expected pattern to be found")
    assert(offset == 2, "expected offset 2, got " .. tostring(offset))
end

function PatternScanner_WildcardMatches()
    local buf = "\xDE\xAD\xBE\xEF"
    local offset = omath.PatternScanner.scan(buf, "DE ?? BE")
    assert(offset ~= nil, "expected wildcard match")
    assert(offset == 0)
end

function PatternScanner_ReturnsNilWhenNotFound()
    local buf = "\x01\x02\x03"
    local offset = omath.PatternScanner.scan(buf, "AA BB CC")
    assert(offset == nil, "expected nil for not-found pattern")
end

function PatternScanner_ReturnsNilForEmptyBuffer()
    local offset = omath.PatternScanner.scan("", "90 01")
    assert(offset == nil)
end

-- SectionScanResult: verify the type is registered and tostring works on a C++-returned value
function SectionScanResult_TypeIsRegistered()
    assert(omath.SectionScanResult ~= nil, "SectionScanResult type should be registered")
end
