-- @ignore
---@meta

-- Error Handling
---@class ScraperError
---@field kind "network" | "cloudflare" | "rate_limit" | "not_found" | "parse" | "validation" | "internal" | string
---@field message string
---@field retryable boolean
---@field status_code integer?

---@class ScraperResult<T>
---@field ok boolean
---@field value T?
---@field error ScraperError?

-- HTTP and response types
---@class CommonHttp
---@field get fun(self: CommonHttp, url: string, headers?: table<string, string>): HttpResponse
---@field post fun(self: CommonHttp, url: string, body: string, headers?: table<string, string>): HttpResponse
---@field has_cloudflare_protection fun(self: CommonHttp, text: string, status_code?: integer, headers?: table<string, string>): boolean
---@field url_encode fun(self: CommonHttp, s: string): string

---@class HttpResponse
---@field ok boolean
---@field error ScraperError?
---@field text string
---@field status integer
---@field headers table<string, string>
---@field json fun(): any

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
---@field get fun(self: FlareSolverrManager, url: string): HttpResponse
---@field using_flaresolverr fun(self: FlareSolverrManager): boolean

-- Custom scraper helpers
---@class CustomScraper
---@field get_image_url fun(self: CustomScraper, html: string): string
---@field get_text fun(self: CustomScraper, html: string): string
---@field get_url fun(self: CustomScraper, html: string): string
---@field select_elements fun(self: CustomScraper, html: string, selector: string): string[]
---@field select_element fun(self: CustomScraper, html: string, selector: string): string?
---@field try_select_elements fun(self: CustomScraper, html: string, selector: string): ScraperResult<string[]>
---@field try_select_element fun(self: CustomScraper, html: string, selector: string): ScraperResult<string>

-- Utility functions
---@class Utils
---@field sleep fun(ms: number)
---@field raise_error fun(kind: string, message: string, retryable: boolean?)

-- Logging
---@class Log
---@field debug fun(...: any)
---@field info fun(...: any)
---@field warn fun(...: any)
---@field error fun(...: any)

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

---@type Log
log = nil

-- Extend the builtin string/table libraries so the language server won't mark our runtime-added helpers as undefined.
---@class stringlib
---@field split fun(s: string, delimiter: string): string[]
---@field trim fun(s: string): string
---@field trim_start fun(s: string): string
---@field trim_end fun(s: string): string
---@field replace fun(s: string, pattern: string, replacement: string): string
---@field contains fun(s: string, needle: string): boolean
---@field starts_with fun(s: string, prefix: string): boolean
---@field ends_with fun(s: string, suffix: string): boolean
---@field substring_after fun(s: string, delimiter: string): string
---@field substring_before fun(s: string, delimiter: string): string

---@type stringlib
string = string or {}

---@class tablelib
---@field reverse fun(t: table): table

---@type tablelib
table = table or {}
