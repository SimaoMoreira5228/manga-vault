-- @ignore
---@meta

-- HTTP and response types
---@class CommonHttp
---@field get fun(self: CommonHttp, url: string, headers?: table<string, string>): HttpResponse
---@field post fun(self: CommonHttp, url: string, body: string, headers?: table<string, string>): HttpResponse
---@field has_cloudflare_protection fun(self: CommonHttp, text: string, status_code?: integer, headers?: table<string, string>): boolean
---@field url_encode fun(self: CommonHttp, s: string): string

---@class HttpResponse
---@field text string
---@field status integer
---@field headers table<string, string>
---@field json fun(self: HttpResponse): table

-- Headless client and element
---@class HeadlessClient
---@field goto fun(self: HeadlessClient, url: string)
---@field find fun(self: HeadlessClient, selector: string): HeadlessElement?
---@field find_all fun(self: HeadlessClient, selector: string): HeadlessElement[]
---@field close fun(self: HeadlessClient)

---@class HeadlessElement
---@field click fun(self: HeadlessElement)
---@field text fun(self: HeadlessElement): string

-- FlareSolverr
---@class FlareSolverrManager
---@field create_session fun(self: FlareSolverrManager): string
---@field get fun(self: FlareSolverrManager, url: string, session_id?: string): HttpResponse
---@field using_flaresolverr fun(self: FlareSolverrManager): boolean

-- Custom scraper helpers
---@class CustomScraper
---@field get_image_url fun(self: CustomScraper, html: string): string
---@field get_text fun(self: CustomScraper, html: string): string
---@field get_url fun(self: CustomScraper, html: string): string
---@field select_elements fun(self: CustomScraper, html: string, selector: string): string[]
---@field select_element fun(self: CustomScraper, html: string, selector: string): string?

-- Utility functions
---@class Utils
---@field sleep fun(ms: number)

-- Declare the runtime globals (so the server knows they exist)
---@type CommonHttp
http = nil

---@type HeadlessClient
headless_client = nil

---@type FlareSolverrManager
flaresolverr = nil

---@type CustomScraper
scraping = nil

---@type Utils
utils = nil

-- Extend the builtin string/table libraries so the language server won't mark our runtime-added helpers as undefined.
---@class stringlib
---@field split fun(s: string, delimiter: string): string[]
---@field trim fun(s: string): string
---@field trim_start fun(s: string): string
---@field trim_end fun(s: string): string
---@field replace fun(s: string, pattern: string, replacement: string): string

---@type stringlib
string = string or {}

---@class tablelib
---@field reverse fun(t: table): table

---@type tablelib
table = table or {}
