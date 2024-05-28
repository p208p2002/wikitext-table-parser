import sys
from wikitext_table_parser import (
    WikitextTableParser,
    Tokenizer,
    Event,
    get_all_table_special_tokens,
    get_all_cell_text_special_tokens
)

table_tokens = get_all_table_special_tokens()
cell_tokens = get_all_cell_text_special_tokens()

table_tokenizer = Tokenizer(table_tokens)
cell_tokenizer = Tokenizer(cell_tokens)

test_case = open(sys.argv[-1]).read()

parser = WikitextTableParser(table_tokenizer, cell_tokenizer, test_case, True)
print(parser.tokens)

while (len(parser.tokens) > 0):
    parser.step()

for event in parser.event_log_queue:
    if isinstance(event, Event.TableStart):
        pass
    elif isinstance(event, Event.TableStyle):
        print("table style:", event.text)
    elif isinstance(event, Event.TableEnd):
        pass
    elif isinstance(event, Event.ColStart):
        print("col type:", event.cell_type)
    elif isinstance(event, Event.ColStyle):
        print("col style:", event.text)
    elif isinstance(event, Event.ColEnd):
        print("col data:", event.text)
    elif isinstance(event, Event.TableCaptionStart):
        pass
    elif isinstance(event, Event.TableCaption):
        print("table caption:", event.text)
    elif isinstance(event, Event.RowStart):
        pass
    elif isinstance(event, Event.RowStyle):
        print("row style:", event.text)
    elif isinstance(event, Event.RowEnd):
        print("-"*20)
    else:
        raise NotImplementedError(event)
