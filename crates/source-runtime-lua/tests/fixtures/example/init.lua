local works = {
	{
		title = "The Weaver's Echo",
		remote_url = "example://works/the-weavers-echo",
	},
}

local chapters = {
	{ title = "Chapter 1: Loom", remote_url = "example://works/the-weavers-echo/1", date = "2026-08-01" },
	{ title = "Chapter 2: Thread", remote_url = "example://works/the-weavers-echo/2", date = "2026-08-08" },
}

local function find(url)
	for _, work in ipairs(works) do
		if work.remote_url == url then
			return work
		end
	end
	return nil
end

function info()
	return {
		id = "example",
		name = "Example",
		version = "0.1.0",
		kind = "novel",
	}
end

function search(query, page)
	if page > 1 then
		return {}
	end
	local out = {}
	for _, work in ipairs(works) do
		if string.find(string.lower(work.title), string.lower(query), 1, true) then
			table.insert(out, work)
		end
	end
	return out
end

function latest(page)
	if page > 1 then
		return {}
	end
	return works
end

function trending(page)
	return latest(page)
end

function fetch_work(url)
	local work = find(url)
	if not work then
		fail("not_found", "unknown work: " .. url)
	end
	return {
		title = work.title,
		remote_url = work.remote_url,
		alternative_names = {},
		authors = { "Example Author" },
		artists = {},
		status = "Ongoing",
		release_date = "2026-08-01",
		description = "A template plugin serving deterministic demo content.",
		genres = { "demo" },
		chapters = chapters,
	}
end

function fetch_chapter(url)
	local number = tonumber(url:match("%d+$")) or 1
	local lines = {}
	for i = 1, 5 do
		table.insert(lines, string.format("<p>Example paragraph %d of chapter %d.</p>", i, number))
	end
	return lines
end

Tests = {
	Test_latest = function()
		assert(#latest(1) == 1, "latest returned nothing")
	end,

	Test_search = function()
		assert(#search("weaver", 1) == 1, "search returned nothing")
		assert(#search("zzz", 1) == 0, "search matched garbage")
	end,

	Test_work_and_chapter = function()
		local details = fetch_work("example://works/the-weavers-echo")
		assert(details.title == "The Weaver's Echo", "title mismatch")
		assert(#details.chapters == 2, "chapters missing")
		local pages = fetch_chapter("example://works/the-weavers-echo/2")
		assert(#pages == 5, "page count mismatch")
		assert(pages[1]:find("chapter 2") ~= nil, "wrong chapter served")
	end,

	Test_fail_not_found = function()
		fetch_work("example://works/ghost")
		error("fetch_work on unknown url must raise")
	end,
}
