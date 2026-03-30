-- trellis-filter.lua
-- Pandoc Lua filter: convert Mermaid fenced code blocks to rendered images.
--
-- Usage:
--   pandoc doc.md --lua-filter trellis-filter.lua -o doc.html
--   pandoc doc.md --lua-filter trellis-filter.lua -o doc.pdf
--
-- The filter invokes `trellis render` for each ```mermaid block it encounters.
-- For HTML/HTML5 output the SVG is embedded inline; for all other formats a
-- temporary PNG file is produced and referenced as a Pandoc Image element.
--
-- Requirements: `trellis` must be on the system PATH.

local function tmpname(ext)
    -- os.tmpname() already provides a unique name; we just append the extension
    return os.tmpname() .. ext
end

local function has_class(el, cls)
    for _, c in ipairs(el.classes) do
        if c == cls then return true end
    end
    return false
end

local function run_trellis(source, out_path, fmt)
    local src_path = tmpname(".mmd")

    local f = io.open(src_path, "w")
    if not f then
        io.stderr:write("[trellis-filter] Cannot create temp file: " .. src_path .. "\n")
        return false
    end
    f:write(source)
    f:close()

    -- Build the command; quote paths to handle spaces
    local cmd = string.format(
        'trellis render "%s" -o "%s" -f %s',
        src_path, out_path, fmt
    )

    local ok = os.execute(cmd)
    os.remove(src_path)

    -- os.execute returns true (Lua 5.2+) or 0 (Lua 5.1) on success
    if ok == true or ok == 0 then
        return true
    end
    io.stderr:write("[trellis-filter] trellis render failed (exit " .. tostring(ok) .. ")\n")
    return false
end

function CodeBlock(el)
    if not has_class(el, "mermaid") then
        return el
    end

    local is_html = FORMAT == "html" or FORMAT == "html5"
    local img_fmt = is_html and "svg" or "png"
    local out_path = tmpname("." .. img_fmt)

    if not run_trellis(el.text, out_path, img_fmt) then
        -- Keep original block if rendering fails
        return el
    end

    if is_html then
        -- Read SVG content and embed inline
        local svg_file = io.open(out_path, "r")
        if svg_file then
            local svg = svg_file:read("*all")
            svg_file:close()
            os.remove(out_path)
            return pandoc.RawBlock("html", svg)
        end
    end

    -- For non-HTML formats return a Pandoc Image element so that Pandoc can
    -- include the file appropriately (e.g. convert PNG into a PDF figure).
    local caption = el.attributes and el.attributes.caption or ""
    local img = pandoc.Image(caption, out_path)
    -- NOTE: out_path is a temp file; it will be cleaned up by the OS after
    -- Pandoc finishes writing its output.
    return pandoc.Para({ img })
end
