// Scroll to the next question once a verdict lands, mirroring the standalone
// HTML trainers. Out-of-band swaps fire this too, so only react to the verdict.
document.body.addEventListener("htmx:afterSwap", (event) => {
  const result = event.target;
  if (!result.classList?.contains("result")) {
    return;
  }

  const current = result.closest(".question");
  if (!current) {
    return;
  }

  const questions = [...document.querySelectorAll(".question")];
  const next = questions[questions.indexOf(current) + 1];
  if (!next) {
    return;
  }

  next.scrollIntoView({ behavior: "smooth", block: "center" });
  next.querySelector(".text-answer")?.focus({ preventScroll: true });
});
