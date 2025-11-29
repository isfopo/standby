#!/bin/bash

# Script to create and set up a Homebrew tap for standby
# Usage: ./setup-homebrew-tap.sh <github-username>

set -e

if [ $# -ne 1 ]; then
    echo "Usage: $0 <github-username>"
    echo "Example: $0 myusername"
    exit 1
fi

USERNAME=$1
REPO_NAME="standby"
TAP_REPO="${REPO_NAME}-homebrew-tap"

echo "Setting up Homebrew tap for $REPO_NAME..."
echo "GitHub username: $USERNAME"
echo "Tap repository: $TAP_REPO"
echo ""

# Create tap repository locally
echo "Creating tap repository structure..."
mkdir -p "$TAP_REPO/Formula"
cd "$TAP_REPO"

# Initialize git repo
git init
git checkout -b main

# Create README for the tap
cat > README.md << EOF
# ${USERNAME}/${TAP_REPO}

Homebrew tap for ${REPO_NAME}.

## Installation

\`\`\`bash
# Add this tap
brew tap ${USERNAME}/${TAP_REPO}

# Install standby
brew install ${REPO_NAME}
\`\`\`

## Updating

The formula will be automatically updated when new releases are published.
EOF

# Create initial formula (will be updated by CI)
cat > Formula/${REPO_NAME}.rb << EOF
class Standby < Formula
  desc "Terminal-based audio monitoring application"
  homepage "https://github.com/${USERNAME}/${REPO_NAME}"
  url "https://github.com/${USERNAME}/${REPO_NAME}/releases/download/v0.1.0/standby-x86_64-apple-darwin.tar.gz"
  sha256 "placeholder-sha256"
  version "0.1.0"

  def install
    bin.install "standby"
  end

  test do
    system "#{bin}/standby", "--help"
  end
end
EOF

# Add and commit files
git add .
git commit -m "Initial commit"

echo ""
echo "Tap repository created locally in ./${TAP_REPO}"
echo ""
echo "Next steps:"
echo "1. Create a new repository on GitHub named '${TAP_REPO}'"
echo "2. Push this local repository to GitHub:"
echo "   cd ${TAP_REPO}"
echo "   git remote add origin https://github.com/${USERNAME}/${TAP_REPO}.git"
echo "   git push -u origin main"
echo ""
echo "3. In your main repository settings, add HOMEBREW_TAP_TOKEN secret"
echo "   - Go to https://github.com/${USERNAME}/${REPO_NAME}/settings/secrets/actions"
echo "   - Add a new secret named HOMEBREW_TAP_TOKEN"
echo "   - Set it to a Personal Access Token with repo permissions"
echo ""
echo "4. For Homebrew Core submission, add HOMEBREW_CORE_TOKEN secret"
echo "   - This requires maintainer access to homebrew-core"
echo ""
echo "The CI workflow will automatically update the formula on releases!"