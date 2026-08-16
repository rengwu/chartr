package sqlite

import (
	"encoding/binary"
	"errors"
	"fmt"
	"math"
	"strings"
)

// A SQLite table is a b-tree keyed by rowid, and that is the whole of what this
// file walks. Interior pages carry (child, key) pairs where key is the largest
// rowid in that child, which is exactly what makes "everything after this rowid"
// a descent rather than a scan: a subtree whose key is at or below the cursor is
// skipped without being read.
//
// That is the property the caller depends on. Several tabs polling the same
// database each read the rows written since their own last poll, and a tab that
// has been open all day pays the same as one opened a minute ago.

const (
	pageInteriorTable = 5
	pageLeafTable     = 13
)

// errStopped unwinds a walk that the caller's own function ended.
var errStopped = errors.New("sqlite: walk stopped")

// Row is one decoded row. A value is nil, int64, float64, string or []byte,
// which is the whole of SQLite's storage classes.
type Row []any

// Text returns column i as a string when it holds text, which is how every
// column this package is used for is stored.
func (r Row) Text(i int) (string, bool) {
	if i < 0 || i >= len(r) {
		return "", false
	}
	s, ok := r[i].(string)
	return s, ok
}

// Int returns column i as an integer when it holds one.
func (r Row) Int(i int) (int64, bool) {
	if i < 0 || i >= len(r) {
		return 0, false
	}
	n, ok := r[i].(int64)
	return n, ok
}

// IsNull reports whether column i is NULL — or is not a column of this row,
// which for a caller's purposes is the same absence.
func (r Row) IsNull(i int) bool { return i < 0 || i >= len(r) || r[i] == nil }

// Table is one table's shape: where its b-tree starts and what its columns are
// called. The column names come from the CREATE statement the database stores,
// so a caller asks for a column by name and a schema that no longer has it fails
// to resolve rather than silently reading a different one.
type Table struct {
	Name string

	root uint32
	cols []string
	// alias is the index of an INTEGER PRIMARY KEY column, which SQLite stores
	// as the rowid rather than in the row, or -1.
	alias int
}

// Column returns the index of a named column.
func (t *Table) Column(name string) (int, bool) {
	for i, col := range t.cols {
		if strings.EqualFold(col, name) {
			return i, true
		}
	}
	return -1, false
}

// Table returns a named table's shape.
func (db *DB) Table(name string) (*Table, bool) {
	t, ok := db.schema[strings.ToLower(name)]
	return t, ok
}

// Rows calls fn for each row of t with a rowid greater than after, in rowid
// order, stopping when fn returns false or when limit rows have been passed.
//
// A limit of zero or less reads everything, which is only ever right for a table
// a caller knows to be small.
func (db *DB) Rows(t *Table, after int64, limit int, fn func(rowid int64, row Row) bool) error {
	if t == nil {
		return ErrUnsupported
	}
	seen := 0
	err := db.walk(t, t.root, after, 0, func(rowid int64, row Row) bool {
		if !fn(rowid, row) {
			return false
		}
		seen++
		return limit <= 0 || seen < limit
	})
	if errors.Is(err, errStopped) {
		return nil
	}
	return err
}

// Row returns one row by rowid.
func (db *DB) Row(t *Table, rowid int64) (Row, bool, error) {
	var (
		found Row
		ok    bool
	)
	err := db.Rows(t, rowid-1, 1, func(id int64, row Row) bool {
		if id == rowid {
			found, ok = row, true
		}
		return false
	})
	if err != nil {
		return nil, false, err
	}
	return found, ok, nil
}

// MaxRowID returns the largest rowid in t, or zero for an empty table. It is how
// a cursor is seated at the end of a table without reading a row of it: the
// descent is down the right-most edge of the tree.
func (db *DB) MaxRowID(t *Table) (int64, error) {
	if t == nil {
		return 0, ErrUnsupported
	}
	page := t.root
	for depth := 0; depth < maxDepth; depth++ {
		data, kind, head, cells, err := db.btreePage(page)
		if err != nil {
			return 0, err
		}
		if kind == pageLeafTable {
			if cells == 0 {
				return 0, nil
			}
			off, err := cellOffset(data, head, cells-1, db.pageSize)
			if err != nil {
				return 0, err
			}
			_, n := uvarint(data[off:])
			if n <= 0 {
				return 0, ErrUnsupported
			}
			rowid, n2 := varint(data[off+n:])
			if n2 <= 0 {
				return 0, ErrUnsupported
			}
			return rowid, nil
		}
		// The right-most pointer of an interior page leads to the largest keys.
		page = binary.BigEndian.Uint32(data[head+8 : head+12])
	}
	return 0, ErrUnsupported
}

// walk descends the tree, visiting every row after the cursor in order.
func (db *DB) walk(t *Table, page uint32, after int64, depth int, fn func(int64, Row) bool) error {
	if depth >= maxDepth {
		return ErrUnsupported
	}
	data, kind, head, cells, err := db.btreePage(page)
	if err != nil {
		return err
	}

	if kind == pageLeafTable {
		for i := 0; i < cells; i++ {
			off, err := cellOffset(data, head, i, db.pageSize)
			if err != nil {
				return err
			}
			size, n := uvarint(data[off:])
			if n <= 0 {
				return ErrUnsupported
			}
			rowid, n2 := varint(data[off+n:])
			if n2 <= 0 {
				return ErrUnsupported
			}
			if rowid <= after {
				continue
			}
			payload, err := db.payload(data, off+n+n2, int64(size))
			if err != nil {
				return err
			}
			row, err := db.decode(t, payload, rowid)
			if err != nil {
				return err
			}
			if !fn(rowid, row) {
				return errStopped
			}
		}
		return nil
	}

	for i := 0; i < cells; i++ {
		off, err := cellOffset(data, head, i, db.pageSize)
		if err != nil {
			return err
		}
		if off+4 > len(data) {
			return ErrUnsupported
		}
		child := binary.BigEndian.Uint32(data[off : off+4])
		key, n := varint(data[off+4:])
		if n <= 0 {
			return ErrUnsupported
		}
		// Every rowid in this subtree is at or below key, so a subtree that ends
		// at or before the cursor holds nothing the caller asked for.
		if key <= after {
			continue
		}
		if err := db.walk(t, child, after, depth+1, fn); err != nil {
			return err
		}
	}
	right := binary.BigEndian.Uint32(data[head+8 : head+12])
	return db.walk(t, right, after, depth+1, fn)
}

// btreePage reads a page and returns it with its header offset, kind and cell
// count. Page 1 carries the database header first, so its tree begins 100 bytes
// in — the one place in the format where a page is not laid out like the others.
func (db *DB) btreePage(n uint32) ([]byte, byte, int, int, error) {
	data, err := db.page(n)
	if err != nil {
		return nil, 0, 0, 0, err
	}
	head := 0
	if n == 1 {
		head = headerSize
	}
	if head+12 > len(data) {
		return nil, 0, 0, 0, ErrUnsupported
	}
	kind := data[head]
	if kind != pageLeafTable && kind != pageInteriorTable {
		// An index page, a WITHOUT ROWID table, or an overflow page reached by
		// a pointer this reader should not have followed.
		return nil, 0, 0, 0, ErrUnsupported
	}
	cells := int(binary.BigEndian.Uint16(data[head+3 : head+5]))
	array := head + 8
	if kind == pageInteriorTable {
		array += 4
	}
	if array+2*cells > len(data) {
		return nil, 0, 0, 0, ErrUnsupported
	}
	return data, kind, head, cells, nil
}

// cellOffset reads the i'th entry of a page's cell pointer array.
func cellOffset(data []byte, head, i, pageSize int) (int, error) {
	array := head + 8
	if data[head] == pageInteriorTable {
		array += 4
	}
	off := int(binary.BigEndian.Uint16(data[array+2*i : array+2*i+2]))
	if off < head || off >= pageSize || off >= len(data) {
		return 0, ErrUnsupported
	}
	return off, nil
}

// payload assembles a cell's bytes, following the overflow chain when the record
// did not fit on its page. The arithmetic is SQLite's own: a payload longer than
// the page can hold keeps a computed prefix in place and puts the rest on a chain
// of overflow pages, each of which begins with the number of the next.
func (db *DB) payload(data []byte, start int, size int64) ([]byte, error) {
	u := int64(db.usable)
	x := u - 35
	local := size
	if size > x {
		m := ((u - 12) * 32 / 255) - 23
		k := m + ((size - m) % (u - 4))
		if k <= x {
			local = k
		} else {
			local = m
		}
	}
	if local < 0 || int64(start)+local > int64(len(data)) {
		return nil, ErrUnsupported
	}
	out := make([]byte, 0, size)
	out = append(out, data[start:int64(start)+local]...)
	if local == size {
		return out, nil
	}

	if int64(start)+local+4 > int64(len(data)) {
		return nil, ErrUnsupported
	}
	next := binary.BigEndian.Uint32(data[int64(start)+local : int64(start)+local+4])
	for remaining := size - local; remaining > 0; {
		if next == 0 {
			return nil, ErrUnsupported // a chain that ends before the payload does
		}
		page, err := db.page(next)
		if err != nil {
			return nil, err
		}
		if int64(len(page)) < u {
			return nil, ErrUnsupported
		}
		take := u - 4
		if take > remaining {
			take = remaining
		}
		out = append(out, page[4:4+take]...)
		remaining -= take
		next = binary.BigEndian.Uint32(page[0:4])
	}
	return out, nil
}

// decode turns one record's bytes into values. The header lists a serial type per
// column, and the bodies follow in the same order; a record whose header does not
// account for exactly what follows is not one this reader believes.
func (db *DB) decode(t *Table, payload []byte, rowid int64) (Row, error) {
	headerSize, n := uvarint(payload)
	if n <= 0 || headerSize > uint64(len(payload)) {
		return nil, ErrUnsupported
	}
	var (
		types []uint64
		at    = n
	)
	for at < int(headerSize) {
		code, n := uvarint(payload[at:])
		if n <= 0 {
			return nil, ErrUnsupported
		}
		types = append(types, code)
		at += n
	}
	if at != int(headerSize) {
		return nil, ErrUnsupported
	}

	row := make(Row, 0, len(types))
	body := int(headerSize)
	for i, code := range types {
		value, size, err := decodeValue(payload, body, code)
		if err != nil {
			return nil, err
		}
		body += size
		if t != nil && i == t.alias {
			// An INTEGER PRIMARY KEY is stored as the rowid, not in the row.
			value = rowid
		}
		row = append(row, value)
	}
	if body != len(payload) {
		return nil, ErrUnsupported
	}
	return row, nil
}

// decodeValue reads one column, returning it and how many bytes it occupied.
func decodeValue(payload []byte, at int, code uint64) (any, int, error) {
	size := 0
	switch {
	case code == 0:
		return nil, 0, nil
	case code >= 1 && code <= 4:
		size = int(code)
	case code == 5:
		size = 6
	case code == 6, code == 7:
		size = 8
	case code == 8:
		return int64(0), 0, nil
	case code == 9:
		return int64(1), 0, nil
	case code == 10 || code == 11:
		return nil, 0, ErrUnsupported // reserved for internal use
	case code%2 == 0:
		size = int((code - 12) / 2)
	default:
		size = int((code - 13) / 2)
	}
	if at+size > len(payload) || size < 0 {
		return nil, 0, ErrUnsupported
	}
	data := payload[at : at+size]

	switch {
	case code >= 1 && code <= 6:
		var n int64
		if len(data) > 0 && data[0]&0x80 != 0 {
			n = -1 // sign-extend
		}
		for _, b := range data {
			n = n<<8 | int64(b)
		}
		return n, size, nil
	case code == 7:
		return math.Float64frombits(binary.BigEndian.Uint64(data)), size, nil
	case code%2 == 0:
		return append([]byte(nil), data...), size, nil
	default:
		return string(data), size, nil
	}
}

// readSchema reads sqlite_schema — the table on page 1 that describes every other
// table — and keeps the ordinary tables from it.
func (db *DB) readSchema() error {
	// sqlite_schema's own shape is fixed by the format.
	schema := &Table{Name: "sqlite_schema", root: 1, alias: -1,
		cols: []string{"type", "name", "tbl_name", "rootpage", "sql"}}

	tables := make(map[string]*Table)
	err := db.Rows(schema, 0, 0, func(_ int64, row Row) bool {
		kind, _ := row.Text(0)
		name, _ := row.Text(1)
		sql, _ := row.Text(4)
		root, ok := row.Int(3)
		if kind != "table" || name == "" || !ok || root <= 0 {
			return true
		}
		cols, alias, ok := parseColumns(sql)
		if !ok {
			return true // a table whose columns cannot be read is not offered
		}
		tables[strings.ToLower(name)] = &Table{
			Name: name, root: uint32(root), cols: cols, alias: alias,
		}
		return true
	})
	if err != nil {
		return fmt.Errorf("sqlite: reading schema: %w", err)
	}
	db.schema = tables
	return nil
}

// parseColumns pulls the column names out of a CREATE TABLE statement, which is
// how a row's positions are given names. It is deliberately small: it splits the
// top-level column list and takes each item's first identifier, skipping the
// table constraints that share that list.
func parseColumns(sql string) ([]string, int, bool) {
	first := strings.IndexByte(sql, '(')
	last := strings.LastIndexByte(sql, ')')
	if first < 0 || last <= first {
		return nil, -1, false
	}

	var (
		cols  []string
		alias = -1
		depth int
		item  strings.Builder
		items []string
	)
	for _, r := range sql[first+1 : last] {
		switch {
		case r == '(':
			depth++
		case r == ')':
			depth--
		case r == ',' && depth == 0:
			items = append(items, item.String())
			item.Reset()
			continue
		}
		item.WriteRune(r)
	}
	items = append(items, item.String())

	for _, raw := range items {
		item := strings.TrimSpace(raw)
		if item == "" {
			continue
		}
		name, rest := identifier(item)
		if name == "" {
			return nil, -1, false
		}
		switch strings.ToUpper(name) {
		case "CONSTRAINT", "PRIMARY", "UNIQUE", "CHECK", "FOREIGN":
			continue // a table constraint, not a column
		}
		// An INTEGER PRIMARY KEY is an alias for the rowid and is not stored in
		// the row itself, so its position has to be filled in from the key.
		upper := strings.ToUpper(strings.Join(strings.Fields(rest), " "))
		if strings.HasPrefix(upper, "INTEGER") && strings.Contains(upper, "PRIMARY KEY") {
			alias = len(cols)
		}
		cols = append(cols, name)
	}
	if len(cols) == 0 {
		return nil, -1, false
	}
	return cols, alias, true
}

// identifier reads the leading name of a column definition, unwrapping whichever
// of SQLite's four quoting styles it was written in.
func identifier(item string) (string, string) {
	var quote byte
	switch item[0] {
	case '`':
		quote = '`'
	case '"':
		quote = '"'
	case '[':
		quote = ']'
	}
	if quote != 0 {
		end := strings.IndexByte(item[1:], quote)
		if end < 0 {
			return "", ""
		}
		return item[1 : 1+end], item[2+end:]
	}
	if i := strings.IndexAny(item, " \t\n\r("); i >= 0 {
		return item[:i], item[i:]
	}
	return item, ""
}

// uvarint reads one of SQLite's big-endian base-128 varints, up to nine bytes.
func uvarint(data []byte) (uint64, int) {
	var v uint64
	for i := 0; i < 8 && i < len(data); i++ {
		b := data[i]
		v = v<<7 | uint64(b&0x7f)
		if b&0x80 == 0 {
			return v, i + 1
		}
	}
	if len(data) < 9 {
		return 0, 0
	}
	return v<<8 | uint64(data[8]), 9
}

// varint is the same value read as signed, which is what a rowid and an integer
// key are.
func varint(data []byte) (int64, int) {
	v, n := uvarint(data)
	return int64(v), n
}
