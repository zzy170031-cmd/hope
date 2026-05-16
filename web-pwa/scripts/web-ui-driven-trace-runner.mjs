import {
  APP_URL,
  FULL16_CASES,
  getCli,
  printJson,
  runWebWorkflowCase,
  summarizeResults,
  TARGETED_CASES,
} from "./web-runner-lib.mjs";

async function main() {
  const cli = getCli();
  const cases = cli.flags.has("full16")
    ? FULL16_CASES
    : cli.flags.has("targeted")
      ? TARGETED_CASES
      : TARGETED_CASES;

  const results = [];
  for (const caseDef of cases) {
    try {
      const result = await runWebWorkflowCase(caseDef, { appUrl: APP_URL });
      results.push(result);
    } catch (error) {
      try {
        const retryResult = await runWebWorkflowCase(caseDef, { appUrl: APP_URL });
        results.push({ ...retryResult, retried_after_error: error instanceof Error ? error.message : String(error) });
      } catch (retryError) {
        results.push({
          ok: false,
          case_id: caseDef.caseId,
          mode: caseDef.mode,
          scene_type: caseDef.sceneType,
          target_duration: caseDef.duration,
          error: retryError instanceof Error ? retryError.message : String(retryError),
          first_error: error instanceof Error ? error.message : String(error),
        });
      }
    }
  }

  printJson({
    mode: cli.flags.has("full16") ? "full16" : "targeted",
    app_url: APP_URL,
    summary: summarizeResults(results),
    results,
  });
}

main()
  .then(() => {
    process.exit(0);
  })
  .catch((error) => {
    printJson({
      ok: false,
      error: error instanceof Error ? error.message : String(error),
    });
    process.exit(1);
  });
