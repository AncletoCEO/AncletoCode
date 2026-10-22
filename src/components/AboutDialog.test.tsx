import { describe, expect, it, vi } from "vitest";
import { render, screen } from "@testing-library/react";
import AboutDialog from "./AboutDialog";

describe("AboutDialog", () => {
  it("declara que es un fork de PandaCode con su enlace", () => {
    render(<AboutDialog onClose={vi.fn()} />);

    expect(screen.getByText(/This is a fork of PandaCode/)).toBeInTheDocument();
    expect(screen.getByRole("link")).toHaveAttribute(
      "href",
      "https://github.com/aicodeking/pandacode",
    );
  });
});
