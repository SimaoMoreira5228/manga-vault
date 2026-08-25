function info()
	return { id = "fixture", name = "Fixture", version = "0.1.0", kind = "novel" }
end

function fetch_chapter(url)
	local response = http.get(url, nil)
	if response.status == 404 then
		fail("not_found", "page not found: " .. url)
	end
end

function latest(page) end
function trending(page) end
function search(query, page) end
function fetch_work(url) end
