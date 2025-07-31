#!/usr/bin/env bash

# ----------------------------------------------------------------------------
# Licensed to the Apache Software Foundation (ASF) under one
# or more contributor license agreements.  See the NOTICE file
# distributed with this work for additional information
# regarding copyright ownership.  The ASF licenses this file
# to you under the Apache License, Version 2.0 (the
# "License"); you may not use this file except in compliance
# with the License.  You may obtain a copy of the License at
#
#    https://www.apache.org/licenses/LICENSE-2.0
#
# Unless required by applicable law or agreed to in writing,
# software distributed under the License is distributed on an
# "AS IS" BASIS, WITHOUT WARRANTIES OR CONDITIONS OF ANY
# KIND, either express or implied.  See the License for the
# specific language governing permissions and limitations
# under the License.
# ----------------------------------------------------------------------------

DIRECTORY="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"

########################################################################################################################
# 0. Check if there are uncommitted changes as these would automatically be committed (local)
########################################################################################################################

if [[ $(git -C "$DIRECTORY" status --porcelain) ]]; then
  # Changes
  echo "❌ There are untracked files or changed files, aborting."
  exit 1
fi

PROJECT_VERSION=$("$DIRECTORY"/mvnw -f "$DIRECTORY"/pom.xml -q -Dexec.executable=echo -Dexec.args="\${project.version}" --non-recursive exec:exec)
RELEASE_VERSION=${PROJECT_VERSION%"-SNAPSHOT"}
IFS='.' read -ra VERSION_SEGMENTS <<< "$RELEASE_VERSION"
NEW_VERSION="${VERSION_SEGMENTS[0]}.${VERSION_SEGMENTS[1]}.$((VERSION_SEGMENTS[2] + 1))-SNAPSHOT"

########################################################################################################################
# 1. Do a simple release-prepare command
########################################################################################################################

if ! docker compose -f "$DIRECTORY/tools/docker-compose.yaml" run releaser \
        bash -c "/ws/mvnw -e -P with-c,with-dotnet,with-go,with-java,with-python,enable-all-checks,update-generated-code -Dmaven.repo.local=/ws/out/.repository release:prepare -DautoVersionSubmodules=true -DreleaseVersion='$RELEASE_VERSION' -DdevelopmentVersion='$NEW_VERSION' -Dtag='v$RELEASE_VERSION'"; then
    echo "❌ Got non-0 exit code from docker compose, aborting."
    exit 1
fi

########################################################################################################################
# 2. Push the changes (local)
########################################################################################################################

if ! git -C "$DIRECTORY" push; then
    echo "❌ Got non-0 exit code from pushing changes to git, aborting."
    exit 1
fi

########################################################################################################################
# 3. Do a simple release-perform command skip signing of artifacts and deploy to local directory (inside the Docker container)
########################################################################################################################

echo "Performing Release:"
#docker compose -f "$DIRECTORY/tools/docker-compose.yaml" build
if ! docker compose -f "$DIRECTORY/tools/docker-compose.yaml" run releaser \
        bash -c "/ws/mvnw -e -Dmaven.repo.local=/ws/out/.repository -DaltDeploymentRepository=snapshot-repo::default::file:/ws/out/.local-artifacts-dir release:perform"; then
    echo "❌ Got non-0 exit code from docker compose, aborting."
    exit 1
fi

########################################################################################################################
# 4. Sign all artifacts
########################################################################################################################

echo "Signing artifacts:"
find $DIRECTORY/out/.local-artifacts-dir -print | grep -E '^((.*\.pom)|(.*\.jar)|(.*\.kar)|(.*\.nar)|(.*-features\.xml)|(.*-cyclonedx\.json)|(.*-cyclonedx\.xml)|(.*-site\.xml)|(.*\.zip))$' | while read -r line ; do
    echo "Processing $line"
    if ! gpg -ab "$line"; then
        echo "❌ Got non-0 exit code from signing artifact, aborting."
        exit 1
    fi
done

########################################################################################################################
# 5. Deploy the artifacts to Nexus
########################################################################################################################

echo "Deploying artifacts:"
if ! "$(DIRECTORY)/mvnw" -f "$(DIRECTORY)/tools/stage.pom" nexus-staging:deploy-staged-repository; then
    cho "❌ Got non-0 exit code from staging artifacts, aborting."
    exit 1
fi

########################################################################################################################
# 6. Close the Nexus staging repository
########################################################################################################################

echo "Closing staging repository:"
if ! "$(DIRECTORY)/mvnw" -f "$(DIRECTORY)/tools/stage.pom" nexus-staging:rc-close; then
    cho "❌ Got non-0 exit code from closing staging repository, aborting."
    exit 1
fi

# Get the url of the closed repo to add that to the VOTE email
DEPLOY_PROPS="$HOME/.m2/repository/.nexus-staging/deploy.properties"
STAGING_REPO_ID=$(grep stagingRepositoryId "$DEPLOY_PROPS" | cut -d= -f2)
NEXUS_URL="https://repository.apache.org/"
STAGING_REPO_URL="$NEXUS_URL/content/repositories/$STAGING_REPO_ID"
echo "✅ Staging repository closed: $STAGING_REPO_URL"

########################################################################################################################
# 7. Prepare a directory for the release candidate
########################################################################################################################

echo "Staging release candidate:"
read -r -p 'Release-Candidate number: ' rcNumber
RELEASE_CANDIDATE="rc$rcNumber"
RELEASE_VERSION=$(find $DIRECTORY/out/.local-artifacts-dir/org/apache/plc4x/plc4x-parent/ -maxdepth 1 -type d | grep -vE 'plc4x-parent/$' | xargs -n 1 basename)
mkdir -p "$DIRECTORY/out/stage/${RELEASE_VERSION}/${RELEASE_CANDIDATE}"
cp "$DIRECTORY/README.md" "$DIRECTORY/out/stage/${RELEASE_VERSION}/${RELEASE_CANDIDATE}/README"
cp "$DIRECTORY/RELEASE_NOTES" "$DIRECTORY/out/stage/${RELEASE_VERSION}/${RELEASE_CANDIDATE}"
cp "$DIRECTORY/out/.local-artifacts-dir/org/apache/plc4x/plc4x-parent/${RELEASE_VERSION}/plc4x-parent-${RELEASE_VERSION}-source-release.zip" "$DIRECTORY/out/stage/${RELEASE_VERSION}/${RELEASE_CANDIDATE}/apache-plc4x-${RELEASE_VERSION}-source-release.zip"
cp "$DIRECTORY/out/.local-artifacts-dir/org/apache/plc4x/plc4x-parent/${RELEASE_VERSION}/plc4x-parent-${RELEASE_VERSION}-source-release.zip.asc" "$DIRECTORY/out/stage/${RELEASE_VERSION}/${RELEASE_CANDIDATE}/apache-plc4x-${RELEASE_VERSION}-source-release.zip.asc"
cp "$DIRECTORY/out/.local-artifacts-dir/org/apache/plc4x/plc4x-parent/${RELEASE_VERSION}/plc4x-parent-${RELEASE_VERSION}-source-release.zip.sha512" "$DIRECTORY/out/stage/${RELEASE_VERSION}/${RELEASE_CANDIDATE}/apache-plc4x-${RELEASE_VERSION}-source-release.zip.sha512"
cp "$DIRECTORY/out/.local-artifacts-dir/org/apache/plc4x/plc4x-parent/${RELEASE_VERSION}/plc4x-parent-${RELEASE_VERSION}-cyclonedx.json" "$DIRECTORY/out/stage/${RELEASE_VERSION}/${RELEASE_CANDIDATE}/apache-plc4x-${RELEASE_VERSION}-cyclonedx.json"
cp "$DIRECTORY/out/.local-artifacts-dir/org/apache/plc4x/plc4x-parent/${RELEASE_VERSION}/plc4x-parent-${RELEASE_VERSION}-cyclonedx.json.asc" "$DIRECTORY/out/stage/${RELEASE_VERSION}/${RELEASE_CANDIDATE}/apache-plc4x-${RELEASE_VERSION}-cyclonedx.json.asc"
cp "$DIRECTORY/out/.local-artifacts-dir/org/apache/plc4x/plc4x-parent/${RELEASE_VERSION}/plc4x-parent-${RELEASE_VERSION}-cyclonedx.xml" "$DIRECTORY/out/stage/${RELEASE_VERSION}/${RELEASE_CANDIDATE}/apache-plc4x-${RELEASE_VERSION}-cyclonedx.xml"
cp "$DIRECTORY/out/.local-artifacts-dir/org/apache/plc4x/plc4x-parent/${RELEASE_VERSION}/plc4x-parent-${RELEASE_VERSION}-cyclonedx.xml.asc" "$DIRECTORY/out/stage/${RELEASE_VERSION}/${RELEASE_CANDIDATE}/apache-plc4x-${RELEASE_VERSION}-cyclonedx.xml.asc"

########################################################################################################################
# 8. Upload the release candidate artifacts to SVN
########################################################################################################################

cd "$DIRECTORY/out/stage/${RELEASE_VERSION}" || exit
svn import "${RELEASE_CANDIDATE}" "https://dist.apache.org/repos/dist/dev/plc4x/${RELEASE_VERSION}/${RELEASE_CANDIDATE}" -m"Staging of ${RELEASE_CANDIDATE} of PLC4X ${RELEASE_VERSION}"

########################################################################################################################
# 9. Make sure the currently used GPG key is available in the KEYS file
########################################################################################################################

ORIGINAL_FILE="$DIRECTORY/out/stage/${RELEASE_VERSION}/${RELEASE_CANDIDATE}/apache-plc4x-${RELEASE_VERSION}-source-release.zip"
SIGNED_FILE="$DIRECTORY/out/stage/${RELEASE_VERSION}/${RELEASE_CANDIDATE}/apache-plc4x-${RELEASE_VERSION}-source-release.zip.asc"

KEYS_URL="https://dist.apache.org/repos/dist/release/plc4x/KEYS"
TEMP_DIR=$(mktemp -d)
KEYS_FILE="$TEMP_DIR/KEYS"
KEYRING="$TEMP_DIR/pubring.kbx"

# Fetch KEYS file
echo "🔽 Downloading KEYS file from $KEYS_URL"
curl -fsSL "$KEYS_URL" -o "$KEYS_FILE"

# Import keys into temporary keyring
echo "🔑 Importing KEYS into temporary GPG keyring"
gpg --no-default-keyring --keyring "$KEYRING" --import "$KEYS_FILE" > /dev/null 2>&1

# Verify the signature
echo "🧾 Verifying signature on $SIGNED_FILE"
if gpg --no-default-keyring --keyring "$KEYRING" --verify "$SIGNED_FILE" "$ORIGINAL_FILE" 2>&1 | grep -q "Good signature"; then
  echo "✅ Signature is valid and signed by a key in the Apache PLC4X KEYS file"
else
  echo "❌ Signature is invalid or the key is not in the Apache PLC4X KEYS file"
    exit 1
fi

# Cleanup
rm -rf "$TEMP_DIR"

########################################################################################################################
# 10. TODO: Send out the [VOTE] and [DISCUSS] emails
########################################################################################################################
