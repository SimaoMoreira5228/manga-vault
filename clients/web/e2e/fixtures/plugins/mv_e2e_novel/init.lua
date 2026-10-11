local TOTAL_WORKS = 20
local CHAPTERS_PER_WORK = 5

local function work_title(n)
	return string.format("Novel Seed %02d", n)
end

local function work_url(n)
	return "e2e-novel://work/" .. n
end

local function chapter_url(n, index)
	return string.format("e2e-novel://work/%d/chapter/%d", n, index)
end

local function work_number(url)
	return tonumber(url:match("/work/(%d+)"))
end

local function summarise(n)
	return { title = work_title(n), remote_url = work_url(n) }
end

function info()
	return {
		id = "mv_e2e_novel",
		name = "E2E Novel",
		version = "1.0.0",
		kind = "novel",
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
		})
	end
	return {
		title = work_title(n),
		remote_url = work_url(n),
		alternative_names = { work_title(n) .. " Alias" },
		authors = { "Seed Writer" },
		artists = {},
		status = "ongoing",
		release_date = "2026-01-01",
		description = "Seeded novel fixture number " .. n .. " for the end-to-end suite.",
		genres = { "seed", "fantasy" },
		chapters = chapters,
	}
end

function fetch_chapter(url)
	local index = tonumber(url:match("chapter/(%d+)")) or 1
	local lines = {}
	for paragraph = 1, 8 do
		table.insert(
			lines,
			string.format(
				"<p>Seeded paragraph %d of chapter %d. The reader settings apply to this text.</p>",
				paragraph,
				index
			)
		)
	end
	return lines
end
