# Install Jason automatically when run as a standalone script
Mix.install([:jason])

defmodule DocInjector do
  def run(files_str, json_path) do
    files = String.split(files_str, " ", trim: true)
    docs_map = load_docs(json_path)

    if map_size(docs_map) == 0 do
      IO.warn("Documentation map is empty. Exiting.")
      System.halt(0)
    end

    Enum.each(files, &inject_file(&1, docs_map))
  end

  defp load_docs(json_path) do
    case File.read(json_path) do
      {:ok, content} ->
        case Jason.decode(content) do
          {:ok, map} when is_map(map) -> map
          {:ok, list} when is_list(list) -> Enum.into(list, %{})
          _ -> %{}
        end

      {:error, _} ->
        IO.warn("Could not read docs file at #{json_path}")
        %{}
    end
  end

  defp inject_file(file_path, docs_map) do
    case File.read(file_path) do
      {:ok, content} ->
        lines = String.split(content, "\n", trim: false)
        updated_lines = inject_docs(lines, docs_map)
        new_content = Enum.join(updated_lines, "\n")

        if new_content != content do
          case File.write(file_path, new_content) do
            :ok -> IO.puts("Updated #{file_path}")
            {:error, reason} -> IO.warn("Failed to write #{file_path}: #{inspect(reason)}")
          end
        end

      {:error, reason} ->
        IO.warn("Could not read file #{file_path}: #{inspect(reason)}")
    end
  end

  defp inject_docs(lines, docs_map) do
    inject_docs_with_lookahead(lines, docs_map, [])
  end

  defp inject_docs_with_lookahead([], _docs_map, acc) do
    Enum.reverse(acc)
  end

  defp inject_docs_with_lookahead([line | rest], docs_map, acc) do
    func_name = extract_function_name(line)

    if func_name && Map.has_key?(docs_map, func_name) && not has_doc_above?(acc) do
      doc_string = docs_map[func_name]
      indent = get_indent(line)
      doc_line = format_doc(doc_string, indent)

      # Inject doc_line BEFORE the current def line
      new_acc = [line, doc_line | acc]
      inject_docs_with_lookahead(rest, docs_map, new_acc)
    else
      inject_docs_with_lookahead(rest, docs_map, [line | acc])
    end
  end

  # Matches 'def foo(', 'def foo do', 'def empty?(', 'def bang!('
  defp extract_function_name(line) do
    case Regex.run(~r/^\s*def\s+([a-z_][a-zA-Z0-9_]*[!?]?)/, line) do
      [_, name] -> name
      _ -> nil
    end
  end

  # Scans backwards through processed lines, skipping blanks and specs
  defp has_doc_above?(acc) do
    acc
    |> Enum.map(&String.trim/1)
    |> Enum.reject(&(&1 == ""))
    |> Enum.reject(&String.starts_with?(&1, "@spec"))
    |> Enum.reject(&String.starts_with?(&1, "@impl"))
    |> case do
      [last_meaningful_line | _] ->
        String.starts_with?(last_meaningful_line, "@doc")
      _ ->
        false
    end
  end

  defp get_indent(line) do
    case Regex.run(~r/^(\s*)/, line) do
      [_, spaces] -> spaces
      _ -> ""
    end
  end

  defp format_doc(doc_string, indent) do
    lines =
      doc_string
      |> String.trim()
      |> String.split("\n")
      |> Enum.map(&(indent <> String.trim_leading(&1)))
      |> Enum.join("\n")

    "#{indent}@doc \"\"\"\n#{lines}\n#{indent}\"\"\""
  end
end

case System.argv() do
  [files_str, json_path] ->
    DocInjector.run(files_str, json_path)

  _ ->
    IO.warn("Usage: elixir scripts/inject_docs.exs '<files>' '<json_path>'")
    System.halt(1)
end
