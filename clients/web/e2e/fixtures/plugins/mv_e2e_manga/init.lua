local TOTAL_WORKS = 40
local CHAPTERS_PER_WORK = 5

local function work_title(n)
	return string.format("Manga Seed %02d", n)
end

local function work_url(n)
	return "e2e-manga://work/" .. n
end

local function chapter_url(n, index)
	return string.format("e2e-manga://work/%d/chapter/%d", n, index)
end

local function work_number(url)
	return tonumber(url:match("/work/(%d+)"))
end

local function summarise(n)
	return { title = work_title(n), remote_url = work_url(n) }
end

function info()
	return {
		id = "mv_e2e_manga",
		name = "E2E Manga",
		version = "1.0.0",
		kind = "manga",
	}
end

function search(query, page)
	if page > 1 then
		return {}
	end
	local matches = {}
	local needle = string.lower(query)
	for n = 1, TOTAL_WORKS do
		if string.find(string.lower(work_title(n)), needle, 1, true) then
			table.insert(matches, summarise(n))
		end
	end
	return matches
end

function latest(page)
	local results = {}
	local first = (page - 1) * 10 + 1
	for n = first, math.min(first + 9, TOTAL_WORKS) do
		table.insert(results, summarise(n))
	end
	return results
end

function trending(page)
	return latest(page)
end

function fetch_work(url)
	local n = work_number(url)
	if n == nil or n < 1 or n > TOTAL_WORKS then
		fail("not_found", "unknown work: " .. url)
	end
	local chapters = {}
	for index = 1, CHAPTERS_PER_WORK do
		table.insert(chapters, {
			title = "Chapter " .. index,
			remote_url = chapter_url(n, index),
			date = string.format("2026-01-%02d", index),
			scanlation_group = "Seed Scans",
		})
	end
	return {
		title = work_title(n),
		remote_url = work_url(n),
		alternative_names = {},
		authors = { "Seed Author" },
		artists = { "Seed Artist" },
		status = "ongoing",
		release_date = "2026-01-01",
		description = "Seeded manga fixture number " .. n .. " for the end-to-end suite.",
		genres = { "seed", "action" },
		chapters = chapters,
	}
end

function fetch_chapter(url)
	local pages = {}
	local index = tonumber(url:match("chapter/(%d+)")) or 1
	for page = 1, 3 do
		table.insert(pages, string.format("http://localhost/seed/chapter-%d/page-%d.png", index, page))
	end
	return pages
end
