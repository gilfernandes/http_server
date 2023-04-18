from xml.dom.minidom import parse

document = parse("mimetypes.xml")

nodes = document.documentElement.childNodes
print(len(nodes))

for node in nodes:
    if len(node.childNodes) > 0:
        if node.childNodes[1].nodeName == "extension" and node.childNodes[3].nodeName == "mime-type":
            extension = node.childNodes[1].childNodes[0].nodeValue
            mime = node.childNodes[3].childNodes[0].nodeValue
            print(f'"{extension}" => ("{mime}", true, true),')



