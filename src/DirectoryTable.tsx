import { Typography } from "@mui/material";
import {
  DataGrid,
  GridColDef,
  GridInputRowSelectionModel,
} from "@mui/x-data-grid";
import { invoke } from "@tauri-apps/api/core";
import { useEffect, useState } from "preact/hooks";

interface FileItem {
  path: string;
  name: string;
  size_bytes: number;
  is_dir: boolean;
  human_readable_size: string;
}

export const DirectoryTable = () => {
  const [path, setPath] = useState<string>(".");
  const [files, setFiles] = useState<FileItem[]>([]);
  const [selectedRows, setSelectedRows] = useState<string[]>([]);

  const fetchDirectory = async (directoryPath: string) => {
    try {
      const data = await invoke<FileItem[]>("scan_directory", {
        path: directoryPath,
      });
      const normalizedPath = directoryPath.startsWith("\\\\?\\")
        ? directoryPath.slice(4)
        : directoryPath;

      setFiles(data);
      setPath(normalizedPath);
    } catch {
      alert("Failed to load directory.");
    }
  };

  const deleteFiles = async () => {
    try {
      await invoke("delete_files", { paths: selectedRows });
      setFiles(files.filter((file) => !selectedRows.includes(file.path)));
      setSelectedRows([]);
    } catch {
      alert("Failed to delete selected files.");
    }
  };

  useEffect(() => {
    invoke<string>("get_executable_directory")
      .then(fetchDirectory)
      .catch(() => alert("Failed to determine initial directory."));
  }, []);

  useEffect(() => {
    const handleKeyDown = (event: KeyboardEvent) => {
      if (event.key === "Backspace" && selectedRows.length > 0) {
        if (confirm("Are you sure you want to delete the selected files?")) {
          deleteFiles();
        }
      }
    };

    window.addEventListener("keydown", handleKeyDown);
    return () => window.removeEventListener("keydown", handleKeyDown);
  }, [selectedRows]);

  const handleRowSelectionChange = (selection: GridInputRowSelectionModel) => {
    if (Array.isArray(selection)) {
      setSelectedRows(selection as string[]);
    } else {
      setSelectedRows([selection.toString()]);
    }
  };

  const columns: GridColDef[] = [
    { field: "name", headerName: "Name", flex: 1, sortable: false },
    { field: "is_dir", headerName: "Type", width: 150, sortable: false },
    {
      field: "human_readable_size",
      headerName: "Size",
      width: 150,
      sortable: false,
    },
  ];

  return (
    <div style={{ padding: "16px" }}>
      <Typography variant="h5">Current Path: {path}</Typography>
      <div style={{ width: "100%", marginTop: "16px" }}>
        <DataGrid
          rows={files}
          columns={columns}
          rowSelectionModel={selectedRows}
          onRowSelectionModelChange={handleRowSelectionChange}
          onRowDoubleClick={({ id }) => {
            const file = files.find((row) => row.path === id);
            if (file?.is_dir) fetchDirectory(file.path);
          }}
          getRowId={(row) => row.path}
          checkboxSelection
          disableColumnMenu
          disableVirtualization
          hideFooter
        />
      </div>
    </div>
  );
};
