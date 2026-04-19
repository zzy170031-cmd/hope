param(
  [Parameter(Mandatory = $true)]
  [string]$Name,

  [string]$BasePath = "E:\\codex\\wt"
)

$repo = "E:\\codex\\hope"
$branch = "codex/$Name"
$target = Join-Path $BasePath ("hope-" + $Name)

if (-not (Test-Path $BasePath)) {
  New-Item -ItemType Directory -Path $BasePath | Out-Null
}

Write-Host "Creating worktree:" $target
git -C $repo worktree add $target -b $branch
