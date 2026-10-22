import type { FileNode } from "../stores/projectsStore";

type ProjectTreeProps = {
  node: FileNode;
};

export default function ProjectTree({ node }: ProjectTreeProps) {
  return (
    <li className="tree-node">
      <span className={node.isDir ? "tree-label directory" : "tree-label"}>
        {node.isDir ? "▸ " : "· "}
        {node.name}
      </span>
      {node.children.length > 0 ? (
        <ul className="tree-children">
          {node.children.map((child) => (
            <ProjectTree key={child.path} node={child} />
          ))}
        </ul>
      ) : null}
    </li>
  );
}
